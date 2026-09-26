#!/usr/bin/env python3
# agnos-qemu-bench.py -- agnsh hosted on a pipe on the real agnos kernel, in QEMU: how it waits.
#
# Builds tests/agnos_hostsh.cyr and agnsh for the agnos target from THIS tree, seeds the driver as
# /bin/agnsh -- so kybernet runs it at boot and nothing is typed -- and the shell under test as
# /bin/agnsh-b, plus an optional baseline as /bin/agnsh-a. The driver hosts each shell on a pipe (the
# shape of a PTY-hosted shell), alternating a and b, and prints HS-* lines this script reads back from
# the serial console. Like scripts/agnos-qemu-test.py it only READS the agnos repo: its kernel and
# rootfs are copied, never modified or restaged.
#
# What it measures (2.0.1 -- agnos issue 2026-09-26 poll-and-yield-loops-should-block):
#   idle   the #99 cpu ticks a pipe-hosted agnsh is charged over 1 s blocked on its empty stdin, and its
#          state (6 = BLOCKED). The issue's gate: 0 % -- this is the one number that is a pass/fail.
#   run    N queued `echo hs` lines through one agnsh, spawn to exit: the foreground launch-and-reap
#          round trip (agnsh's parse, audit and history work included, identically for both shells).
#   pipe   N queued `echo hs | wc` pipelines, the same way: both stages' reaps.
# And what it checks, once per shell after the timings: `agnsh -c <line>`'s exit status against ADR-008 § 4
# (126 refused, 127 nothing to run or not launched, 1 usage) for each agnos launcher -- plain, `&`, `|` and `>` --
# spawned with SPAWN_F_ARGV so the line keeps its spaces. The typed session in scripts/agnos-qemu-test.py
# cannot read a `-c` status, so this is the only place one is checked on agnos. A check also wants the
# shell's own stderr message, so a right status for the wrong reason fails. Fixtures: /bin/notelf (not an
# ELF: every launch of it fails with NOEXEC) and /hs-link (a symlink `>` must refuse to open).
#
# Usage:
#   python3 scripts/agnos-qemu-bench.py                                  # this tree only
#   AGNSH_BASELINE=/path/agnsh_agnos python3 scripts/agnos-qemu-bench.py # a/b against a baseline build
#   AGNOS_QEMU_SMP=4 python3 scripts/agnos-qemu-bench.py                 # on 4 CPUs
# Env: AGNOS_ROOT (default ../agnos), GNOBOOT_ROOT (default ../gnoboot), AGNOS_QEMU_TCG=1 forces TCG.
# ⚠ Timing under QEMU: compare a and b from the SAME boot only (they alternate for that reason), and
# check /proc/loadavg first -- a loaded host reads slow. Exit 0 when every shell idled at <= 5 ticks
# in state 6, every batch exited 0 and every `-c` check of b (this tree) held; the timings, and a
# baseline's `-c` statuses, are reported, not gated.
import os, re, shutil, statistics, subprocess, sys, time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
AGNOS_ROOT = os.path.abspath(os.environ.get("AGNOS_ROOT", os.path.join(REPO, "../agnos")))
GNOBOOT = os.path.abspath(os.environ.get("GNOBOOT_ROOT", os.path.join(REPO, "../gnoboot"))) + "/build/BOOTX64.EFI"
KERNEL = os.path.join(AGNOS_ROOT, "build/agnos")
ROOTFS = os.path.join(AGNOS_ROOT, "build/rootfs")
WORK = os.path.join(REPO, "build/agnos-qemu-bench")    # build/ is gitignored
IMG = os.path.join(WORK, "agnsh-bench.img")
SEED = os.path.join(WORK, "seed")
SER = os.path.join(WORK, "serial.log")
PART_OFFSET = 33 * 1048576
PART_BLOCKS = (67 * 1048576) // 4096
EXT2_FEATURES = "^resize_inode,^dir_index,^metadata_csum,^64bit,^uninit_bg"
SMP = int(os.environ.get("AGNOS_QEMU_SMP", "1"))
IDLE_TICKS_MAX = 5                                      # agnos wait-ring3 P2c's bound for a parked caller


def p(*a):
    print(*a, flush=True)


def die(msg):
    p("FAIL:", msg)
    sys.exit(1)


def sh(cmd):
    r = subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    if r.returncode != 0:
        die("build step: " + cmd + "\n" + r.stderr.decode("latin1")[:400])


def build(src, out):
    env = dict(os.environ, CYRIUS_NO_WARN_SHADOW_LIB="1")
    r = subprocess.run(["cyrius", "build", "--agnos", src, out], cwd=REPO, env=env,
                       stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if r.returncode != 0 or not os.path.exists(out):
        die("cyrius build --agnos " + src + ":\n" + r.stdout.decode("latin1")[-600:])


for path in (GNOBOOT, KERNEL, ROOTFS):
    if not os.path.exists(path):
        die("missing " + path + " (build agnos and gnoboot first)")
OVMF_CODE = next((c for c in ("/usr/share/edk2/x64/OVMF_CODE.4m.fd", "/usr/share/edk2/x64/OVMF_CODE.fd",
                               "/usr/share/OVMF/OVMF_CODE.fd", "/usr/share/OVMF/OVMF_CODE_4M.fd")
                  if os.path.exists(c)), None)
OVMF_VARS = next((c for c in ("/usr/share/edk2/x64/OVMF_VARS.4m.fd", "/usr/share/edk2/x64/OVMF_VARS.fd",
                               "/usr/share/OVMF/OVMF_VARS.fd", "/usr/share/OVMF/OVMF_VARS_4M.fd")
                  if os.path.exists(c)), None)
if not OVMF_CODE or not OVMF_VARS:
    die("OVMF not found")

shutil.rmtree(WORK, ignore_errors=True)
os.makedirs(WORK)
build("tests/agnos_hostsh.cyr", os.path.join(WORK, "hostsh"))
build("src/agnsh.cyr", os.path.join(WORK, "agnsh-b"))
baseline = os.environ.get("AGNSH_BASELINE")
if baseline and not os.path.exists(baseline):
    die("AGNSH_BASELINE: no such file " + baseline)
kver = re.search(rb"AGNOS kernel v([0-9.]+)", open(KERNEL, "rb").read())
p("agnos kernel:", kver.group(1).decode() if kver else "(version string not found)", "-", KERNEL)
p("b (this tree):", os.path.join(WORK, "agnsh-b"))
p("a (baseline): ", os.path.abspath(baseline) if baseline else "(none)")

subprocess.run(["cp", "-a", ROOTFS, SEED], check=True)
seeded = [("hostsh", "bin/agnsh"), ("agnsh-b", "bin/agnsh-b")]
for src, dst in seeded:
    shutil.copyfile(os.path.join(WORK, src), os.path.join(SEED, dst))
    os.chmod(os.path.join(SEED, dst), 0o755)
if baseline:
    shutil.copyfile(baseline, os.path.join(SEED, "bin/agnsh-a"))
    os.chmod(os.path.join(SEED, "bin/agnsh-a"), 0o755)
# The `-c` checks' fixtures: a program that passes agnsh's existence probe and fails every launch with
# NOEXEC (agnos pipeline-smoke's /bin/notelf), and a symlink that `>` must refuse to open (AO_NOFOLLOW).
with open(os.path.join(SEED, "bin/notelf"), "w") as f:
    f.write("not an ELF: every launch of it must fail\n")
os.chmod(os.path.join(SEED, "bin/notelf"), 0o755)
with open(os.path.join(SEED, "hs-target.txt"), "w") as f:
    f.write("HS-TARGET-ORIGINAL\n")
os.symlink("/hs-target.txt", os.path.join(SEED, "hs-link"))
for stale in (".agnsh_audit.log", ".agnsh_history"):
    try:
        os.unlink(os.path.join(SEED, stale))
    except FileNotFoundError:
        pass
sh(f"dd if=/dev/zero of={IMG} bs=1M count=128 status=none")
sh(f"parted -s {IMG} mklabel gpt mkpart ESP fat32 1MiB 33MiB set 1 esp on mkpart agnos-fs ext2 33MiB 100MiB")
sh(f"sgdisk -t 2:8300 {IMG} >/dev/null")
sh(f"mformat -i {IMG}@@1048576 -F")
sh(f"mmd -i {IMG}@@1048576 ::EFI ::EFI/BOOT ::boot")
sh(f"mcopy -i {IMG}@@1048576 {GNOBOOT} ::EFI/BOOT/BOOTX64.EFI")
sh(f"mcopy -i {IMG}@@1048576 {KERNEL} ::boot/agnos")
sh(f"mkfs.ext2 -F -q -L AGNOS-HB -b 4096 -m 0 -O {EXT2_FEATURES} -d {SEED} -E offset={PART_OFFSET} {IMG} {PART_BLOCKS}")
accel = ["-cpu", "max"]
if SMP > 1:
    accel = ["-accel", "tcg,thread=multi", "-cpu", "max"]
if os.environ.get("AGNOS_QEMU_TCG") != "1" and os.access("/dev/kvm", os.R_OK | os.W_OK):
    accel = ["-enable-kvm", "-cpu", "host"]
p("accelerator: ", " ".join(accel), f"-smp {SMP}")
with open("/proc/loadavg") as f:
    p("host load:   ", f.read().split()[0], f"({os.cpu_count()} CPUs)")


def ser():
    try:
        return open(SER, "rb").read().decode("latin1").replace("\r", "")
    except OSError:
        return ""


def stop(q):
    if q.poll() is None:
        q.terminate()
        try:
            q.wait(timeout=3)
        except subprocess.TimeoutExpired:
            q.kill()


# ⚠ A FAILED FIRMWARE HAND-OFF IS VOID, NOT A RESULT: the kernel never ran. OVMF sometimes fails
# gnoboot's ExitBootServices and drops to its boot menu for good; agnos's smokes retry with fresh NVRAM
# (qemu-dwell.sh's QEMU_DWELL_VOID), and so does this. The disk is untouched when the kernel never ran.
VOID = re.compile(r"gnoboot: fail @|BdsDxe: failed to load|BootManagerMenuApp|Please select boot device")
for attempt in range(1, 7):
    shutil.copyfile(OVMF_VARS, os.path.join(WORK, "vars.fd"))
    open(SER, "w").close()
    qemu = subprocess.Popen([
        "qemu-system-x86_64", "-machine", "q35", "-m", "512M", *accel, "-smp", str(SMP),
        "-drive", f"if=pflash,format=raw,readonly=on,file={OVMF_CODE}",
        "-drive", f"if=pflash,format=raw,file={WORK}/vars.fd",
        "-drive", f"file={IMG},format=raw,if=none,id=disk0",
        "-device", "nvme,drive=disk0,serial=AGNOS-HB",
        "-serial", f"file:{SER}", "-display", "none", "-no-reboot",
    ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    void = False
    try:
        deadline = time.time() + 600
        while time.time() < deadline:
            out = ser()
            if "HS-DONE" in out or "HS-FAIL" in out or qemu.poll() is not None:
                break
            if "AGNOS kernel v" not in out and VOID.search(out):
                void = True
                break
            time.sleep(1.0)
        time.sleep(1.0)
    finally:
        stop(qemu)
    if not void:
        break
    p(f"  (VOID attempt {attempt}: the firmware never handed off -- retrying with fresh NVRAM)")
else:
    die("the firmware never handed off in 6 attempts (infrastructure, not agnsh)")

out = ser()
if "HS-START" not in out:
    p(out[-800:])
    die("the driver never started (serial log: " + SER + ")")
if "HS-DONE" not in out:
    p(out[-800:])
    die("the driver did not finish (serial log: " + SER + ")")

rows = {}                                   # (kind, shell) -> [dict]
cchk = {}                                   # (shell, tag) -> (dict, the -c line)
cmsg = {}                                   # (shell, tag) -> the first line of a failed check's stderr
cend = {}                                   # shell -> the checks its driver says it ran
ctags = []                                  # tags in the driver's order
for line in out.split("\n"):
    m = re.match(r"\s*HS-(IDLE|RUN|PIPE) (\S+)((?: \w+=-?\d+)*)\s*$", line)
    if m:
        kv = {k: int(v) for k, v in re.findall(r"(\w+)=(-?\d+)", m.group(3))}
        rows.setdefault((m.group(1), m.group(2)), []).append(kv)
        continue
    m = re.match(r"\s*HS-C (\S+) (\S+)((?: \w+=-?\d+)*) :: (.*?)\s*$", line)
    if m:
        kv = {k: int(v) for k, v in re.findall(r"(\w+)=(-?\d+)", m.group(3))}
        cchk[(m.group(1), m.group(2))] = (kv, m.group(4))
        if m.group(2) not in ctags:
            ctags.append(m.group(2))
        continue
    m = re.match(r"\s*HS-CMSG (\S+) (\S+) ?(.*?)\s*$", line)
    if m:
        cmsg[(m.group(1), m.group(2))] = m.group(3)
        continue
    m = re.match(r"\s*HS-CEND (\S+) n=(\d+)\s*$", line)
    if m:
        cend[m.group(1)] = int(m.group(2))

ok = True
shells = [s for s in ("/bin/agnsh-a", "/bin/agnsh-b") if ("IDLE", s) in rows]
if "/bin/agnsh-b" not in shells:
    die("no HS-IDLE line for /bin/agnsh-b")
p("")
p(f"{'shell':<14}{'idle ticks':>11}{'state':>7}   {'run: us per line (median, min-max)':<38}"
  f"{'pipe: us per line (median, min-max)'}")
for s in shells:
    idle = rows[("IDLE", s)][0]
    held = idle.get("ticks", 99) <= IDLE_TICKS_MAX and idle.get("state") == 6 and idle.get("exit") == 0
    cols = []
    for kind in ("RUN", "PIPE"):
        rs = rows.get((kind, s), [])
        if not rs or any(r.get("exit") != 0 for r in rs):
            held = False
        per = [r["us"] / r["n"] for r in rs if r.get("n")]
        cols.append(f"{statistics.median(per):8.0f}  ({min(per):.0f}-{max(per):.0f}, {len(per)} runs)"
                    if per else "(none)")
    ok = ok and held
    p(f"{s[5:]:<14}{idle.get('ticks', -1):>11}{idle.get('state', -1):>7}   {cols[0]:<38}{cols[1]}"
      + ("" if held else "   <- FAIL"))

# `-c` exit statuses (ADR-008 § 4). b is gated; a baseline is shown beside it, so an a/b run against an
# older build reads as before / after. A status whose stderr lacked the expected message shows as `127?`.
B = "/bin/agnsh-b"
have_ca = any(s == "/bin/agnsh-a" for s, _ in cchk)
p("")
p("agnsh -c exit status (ADR-008 § 4: 126 refused, 127 nothing to run or not launched, 1 usage)")
p(f"  {'check':<14}" + (f"{'a':>6}" if have_ca else "") + f"{'b':>6}{'want':>6}   -c line")


def cell(shell, tag):
    got = cchk.get((shell, tag))
    if got is None:
        return "-"
    return str(got[0].get("exit", "?")) + ("" if got[0].get("msg") == 1 else "?")


c_held = 0
for tag in ctags:
    got = cchk.get((B, tag))
    want = got[0].get("want") if got else None
    line = (got or cchk.get(("/bin/agnsh-a", tag)))[1]
    shown = line if len(line) <= 40 else line[:22] + "..." + line[-12:]
    held = got is not None and got[0].get("exit") == want and got[0].get("msg") == 1
    c_held += held
    p(f"  {tag:<14}" + (f"{cell('/bin/agnsh-a', tag):>6}" if have_ca else "") + f"{cell(B, tag):>6}"
      + f"{'-' if want is None else want:>6}   {shown}" + ("" if held else "   <- FAIL"))
    if not held and (B, tag) in cmsg:
        p(f"  {'':<14}   b said: {cmsg[(B, tag)]}")
c_rows = sum(1 for s, _ in cchk if s == B)
if c_rows == 0 or cend.get(B) != c_rows or c_held != len(ctags):
    ok = False
    if cend.get(B) != c_rows:
        p(f"  b: the driver reported {cend.get(B)} checks and printed {c_rows}")
p("")
p("agnos-qemu-bench: " + (f"PASS (idle gate held; {c_held}/{len(ctags)} -c statuses held; timings reported)"
                          if ok else "FAIL"))
sys.exit(0 if ok else 1)
