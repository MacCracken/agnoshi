#!/usr/bin/env python3
# agnos-qemu-test.py -- agnsh on the real agnos kernel, in QEMU: the exec surface and its audit trail.
#
# Builds agnsh for the agnos target from THIS tree, seeds it into a COPY of agnos's build/rootfs,
# boots agnos (gnoboot + build/agnos) in QEMU, types at the agnsh prompt through an emulated xHCI
# keyboard, and checks what agnsh DID against what it RECORDED. The audit log is read back twice:
# in the guest with `owl -p` (byte-identical cat, over the serial console) and, after agnsh's own
# `poweroff`, from the disk image itself (debugfs). The agnos repo is only READ: its kernel and
# rootfs are copied, never modified or restaged.
#
# ⛔ WHY THIS EXISTS. agnsh is agnos's shell -- kybernet execs it on every boot, and agnos's own
# harnesses (agnos/scripts/harness/) exercise its launch paths. What this repo lacked was a test of
# its own there: CI builds the agnos target and cannot run it, and nothing in either repo read the
# audit log agnsh writes on agnos back. Its first run (1.9.14) found that log overwriting itself:
# agnos ignores AO_APPEND, and a session's whole trail came back as its last record plus the torn
# tails of longer ones.
#
# Usage:
#   python3 scripts/agnos-qemu-test.py                 # build agnsh from this tree, then test it
#   AGNSH_AGNOS=/path/agnsh python3 scripts/...        # test a prebuilt agnos-target binary
#   AGNOS_QEMU_SMP=4 python3 scripts/...               # the same session on 4 CPUs
# Env: AGNOS_ROOT (default ../agnos), GNOBOOT_ROOT (default ../gnoboot),
#      AGNOS_QEMU_TCG=1 forces TCG instead of KVM, AGNOS_QEMU_SMP (default 1) sets the CPU count,
#      SLEEP_SAFETY_TICKS caps a sleeper's life.
# Exit 0 only when every check was evaluated and held (the verdict starts at FAIL).
#
# ⛔ A SHELL THAT STOPS RETURNING ITS PROMPT ENDS THE RUN. Every later line would be typed into a
# wedged shell, so the harness says which line did it and exits FAIL rather than timing out forty
# more checks. 2.0.1's pipeline cases exist to catch exactly that (agnos pipeline-smoke's `early` and
# `s2fail`): a pipeline whose consumer is gone must still come back.
#
# Lessons carried over from agnos's harnesses (agnos/scripts/harness/README.md):
#   - the first keystroke of a session is swallowed, so a bare Enter goes first;
#   - oracles are agnsh's OWN output or the log it wrote, never the echo of what was typed;
#   - a harness nobody has watched fail proves nothing: run it against the unfixed code too.
import os, re, shutil, socket, struct, subprocess, sys, time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
AGNOS_ROOT = os.path.abspath(os.environ.get("AGNOS_ROOT", os.path.join(REPO, "../agnos")))
GNOBOOT = os.path.abspath(os.environ.get("GNOBOOT_ROOT", os.path.join(REPO, "../gnoboot"))) + "/build/BOOTX64.EFI"
KERNEL = os.path.join(AGNOS_ROOT, "build/agnos")
ROOTFS = os.path.join(AGNOS_ROOT, "build/rootfs")
SMP = int(os.environ.get("AGNOS_QEMU_SMP", "1"))
# build/ is gitignored; a multi-CPU run gets its own directory, so the two can run side by side.
WORK = os.path.join(REPO, "build/agnos-qemu" + ("" if SMP == 1 else f"-smp{SMP}"))
IMG = os.path.join(WORK, "agnsh-test.img")
SEED = os.path.join(WORK, "seed")
SER = os.path.join(WORK, "serial.log")
MON = os.path.join(WORK, "mon.sock")
PART_OFFSET = 33 * 1048576
PART_BLOCKS = (67 * 1048576) // 4096
EXT2_FEATURES = "^resize_inode,^dir_index,^metadata_csum,^64bit,^uninit_bg"
AUDIT = "/.agnsh_audit.log"
# The sleepers run until the harness creates /stop (checked every ~0.25 s), with a ~20 min safety
# stop so a lost `touch` cannot hang the run. Neither an iteration count nor a fixed wall time
# works: typing on agnos in QEMU runs at roughly a keystroke a second, so eight `sleeper &` lines
# take minutes, and any sleeper that exits early is reaped -- freeing a slot and turning the
# job-cap check into a coin toss. Tick counts assume ~3.2 GHz; being 2x off does not matter.
SAFETY_TICKS = int(os.environ.get("SLEEP_SAFETY_TICKS", str(3840 * 10**9)))
CHECK_TICKS = 800 * 10**6
JOB_CAP = 8                                             # run_agnos.cyr's background-job table
CERT = "/etc/ssl/cert.pem"                              # agnos's rootfs ships it: 185 KB, 45 rings' worth


def p(*a):
    print(*a, flush=True)


def die(msg):
    p("FAIL:", msg)
    sys.exit(1)


def sh(cmd):
    r = subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    if r.returncode != 0:
        die("build step: " + cmd + "\n" + r.stderr.decode("latin1")[:400])


# Assembled once with GNU as (source below), embedded as bytes so the harness needs no binutils:
#     rdtsc; shl rdx,32; or rax,rdx; mov r12,rax; mov r14,rax      ; start, next look
#     movabs r13,SAFETY ; movabs r15,CHECK                            ; immediates at +17 / +27
#   loop: mov eax,44; syscall                                          ; sched_yield
#     rdtsc; shl rdx,32; or rax,rdx; mov rbx,rax                       ; now
#     sub rax,r12; cmp rax,r13; jae done                               ; safety stop
#     cmp rbx,r14; jb loop; lea r14,[rbx+r15]                          ; look every CHECK ticks
#     mov eax,7; lea rdi,[rip+stopname]; mov esi,5; xor edx,edx; syscall   ; open("/stop",5,0)
#     test rax,rax; js loop; mov rdi,rax; mov eax,6; syscall           ; found: close(fd)
#   done: mov eax,1; mov edi,1; lea rsi,[rip+msg]; mov edx,13; syscall  ; write SLEEPER-DONE
#     xor eax,eax; xor edi,edi; syscall; spin: jmp spin                ; exit(0)
#   stopname: "/stop"   msg: "SLEEPER-DONE\n"
SLEEPER_BLOB = bytes.fromhex(
    "0f3148c1e2204809d04989c44989c649bd111111111111111149bf2222222222222222"
    "b82c0000000f050f3148c1e2204809d04889c34c29e04c39e8732d4c39f372e04e8d343b"
    "b807000000488d3d38000000be0500000031d20f054885c078c24889c7b8060000000f05"
    "b801000000bf01000000488d3514000000ba0d0000000f0531c031ff0f05ebfe"
    "2f73746f70534c45455045522d444f4e450a")


def write_elf(path, code):
    """`code` as a static ELF64 at 0x400078: one R+X PT_LOAD covering the file."""
    filesz = 120 + len(code)
    eh = bytearray(64)
    eh[0:7] = b"\x7fELF\x02\x01\x01"
    struct.pack_into("<HHIQQ", eh, 16, 2, 0x3E, 1, 0x400078, 64)   # ET_EXEC, x86-64, entry, phoff
    struct.pack_into("<HHH", eh, 52, 64, 56, 1)                     # ehsize, phentsize, phnum
    ph = bytearray(56)
    struct.pack_into("<IIQQQQQQ", ph, 0, 1, 5, 0, 0x400000, 0x400000, filesz, filesz, 0x1000)
    with open(path, "wb") as f:
        f.write(bytes(eh) + bytes(ph) + bytes(code))
    os.chmod(path, 0o755)


def build_sleeper(path):
    """A static ELF64 that waits -- yielding, near-zero CPU -- until /stop exists, then writes
    SLEEPER-DONE and exits 0. Not sleep_ms#41: that holds the CPU on agnos (agnos issue
    2026-09-23-sleep-ms-holds-the-cpu). A busy-counting sleeper was tried first and starved the
    machine: with eight on one vCPU, keystrokes were dropped mid-command ("sleeper" arrived as
    "sleer"). agnos leaves CR4.TSD clear, so rdtsc is legal in ring 3.
    ⚠ Loop state lives in rbx/r12-r15: an agnos syscall clobbers rcx, rdx, rsi, rdi and r8-r11
    and preserves only rbx, rbp and r12-r15 (agnos kernel/arch/x86_64/syscall_hw.cyr)."""
    code = bytearray(SLEEPER_BLOB)
    assert code[17:25] == b"\x11" * 8 and code[27:35] == b"\x22" * 8
    code[17:25] = struct.pack("<Q", SAFETY_TICKS)
    code[27:35] = struct.pack("<Q", CHECK_TICKS)
    write_elf(path, code)


# mov eax,2; syscall (getpid) · mov edi,eax; mov esi,9; mov eax,16; syscall (kill self, SIGKILL) · jmp $
# agnos lets a process signal itself (proc_may_signal), and a kill of a running process ends it at the
# kill's own syscall exit (boundary B1), so the jmp never runs. agnos 1.57.7 reports the death as the
# wait status 0x100 | 9; agnsh must report 128 + 9 (ADR-008 § 4).
SELFKILL_CODE = bytes.fromhex("b8020000000f0589c7be09000000b8100000000f05ebfe")


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

# ---- the agnsh under test ----
shutil.rmtree(WORK, ignore_errors=True)
os.makedirs(WORK)
agnsh = os.environ.get("AGNSH_AGNOS")
if agnsh:
    agnsh = os.path.abspath(agnsh)
else:
    agnsh = os.path.join(WORK, "agnsh_agnos")
    env = dict(os.environ, CYRIUS_NO_WARN_SHADOW_LIB="1")
    r = subprocess.run(["cyrius", "build", "--agnos", "src/agnsh.cyr", agnsh], cwd=REPO, env=env,
                       stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if r.returncode != 0:
        die("cyrius build --agnos:\n" + r.stdout.decode("latin1")[-600:])
if not os.path.exists(agnsh):
    die("no agnsh binary at " + agnsh)
kver = re.search(rb"AGNOS kernel v([0-9.]+)", open(KERNEL, "rb").read())
p("agnsh under test:", agnsh)
p("agnos kernel:    ", kver.group(1).decode() if kver else "(version string not found)", "-", KERNEL)

# ---- image: gnoboot + kernel on the ESP; a copy of agnos's rootfs, plus fixtures, as ext2 ----
subprocess.run(["cp", "-a", ROOTFS, SEED], check=True)
shutil.copyfile(agnsh, os.path.join(SEED, "bin/agnsh"))
os.chmod(os.path.join(SEED, "bin/agnsh"), 0o755)
for stale in (".agnsh_audit.log", ".agnsh_history"):          # every run starts from no state
    try:
        os.unlink(os.path.join(SEED, stale))
    except FileNotFoundError:
        pass
build_sleeper(os.path.join(SEED, "bin/sleeper"))
write_elf(os.path.join(SEED, "bin/selfkill"), SELFKILL_CODE)
# Passes agnsh's existence probe and fails spawn_path#43 with NOEXEC: a pipeline stage 2 that
# cannot start (agnos pipeline-smoke's /bin/notelf).
with open(os.path.join(SEED, "bin/notelf"), "w") as f:
    f.write("not an ELF: stage 2 of a pipeline must fail to spawn\n")
os.chmod(os.path.join(SEED, "bin/notelf"), 0o755)
if not os.path.exists(os.path.join(SEED, CERT.lstrip("/"))):
    die(CERT + " is not in " + ROOTFS + " (the streaming and early-exit pipelines need > 4080 B)")
# What the streaming pipeline must count: `grep .` passes every non-empty line, newline included —
# the host's own `grep . | wc -c` over the same file.
with open(os.path.join(SEED, CERT.lstrip("/")), "rb") as f:
    CERT_NONEMPTY = sum(len(l) + 1 for l in f.read().split(b"\n") if l)
with open(os.path.join(SEED, "redir-target.txt"), "w") as f:  # a planted symlink points here
    f.write("REDIRECT-TARGET-ORIGINAL\n")
os.symlink("/redir-target.txt", os.path.join(SEED, "redir-link"))
sh(f"dd if=/dev/zero of={IMG} bs=1M count=128 status=none")
sh(f"parted -s {IMG} mklabel gpt mkpart ESP fat32 1MiB 33MiB set 1 esp on mkpart agnos-fs ext2 33MiB 100MiB")
sh(f"sgdisk -t 2:8300 {IMG} >/dev/null")
sh(f"mformat -i {IMG}@@1048576 -F")
sh(f"mmd -i {IMG}@@1048576 ::EFI ::EFI/BOOT ::boot")
sh(f"mcopy -i {IMG}@@1048576 {GNOBOOT} ::EFI/BOOT/BOOTX64.EFI")
sh(f"mcopy -i {IMG}@@1048576 {KERNEL} ::boot/agnos")
sh(f"mkfs.ext2 -F -q -L AGNOS-BG -b 4096 -m 0 -O {EXT2_FEATURES} -d {SEED} -E offset={PART_OFFSET} {IMG} {PART_BLOCKS}")

accel = ["-cpu", "max"]
if SMP > 1:
    accel = ["-accel", "tcg,thread=multi", "-cpu", "max"]   # agnos smoke_accel's multi-CPU TCG
if os.environ.get("AGNOS_QEMU_TCG") != "1" and os.access("/dev/kvm", os.R_OK | os.W_OK):
    accel = ["-enable-kvm", "-cpu", "host"]
p("accelerator:     ", " ".join(accel), f"-smp {SMP}")

# ⚠ A FAILED FIRMWARE HAND-OFF IS VOID, NOT A FAILURE OF AGNSH: OVMF sometimes fails gnoboot's
# ExitBootServices and drops to its boot menu for good, and the kernel never runs. agnos's smokes
# retry with fresh NVRAM (qemu-dwell.sh's QEMU_DWELL_VOID); so does this, up to six boots. The disk
# is untouched when the kernel never ran. Until 2.0.1 a void boot was scored as agnsh never
# reaching its banner.
VOID = re.compile(r"gnoboot: fail @|BdsDxe: failed to load|BootManagerMenuApp|Please select boot device")
for attempt in range(1, 7):
    shutil.copyfile(OVMF_VARS, os.path.join(WORK, "vars.fd"))
    open(SER, "w").close()
    if os.path.exists(MON):
        os.unlink(MON)
    qemu = subprocess.Popen([
        "qemu-system-x86_64", "-machine", "q35", "-m", "512M", *accel, "-smp", str(SMP),
        "-drive", f"if=pflash,format=raw,readonly=on,file={OVMF_CODE}",
        "-drive", f"if=pflash,format=raw,file={WORK}/vars.fd",
        "-drive", f"file={IMG},format=raw,if=none,id=disk0",
        "-device", "nvme,drive=disk0,serial=AGNOS-BG",
        "-device", "qemu-xhci,id=xhci", "-device", "usb-kbd,bus=xhci.0",
        "-serial", f"file:{SER}", "-display", "none", "-no-reboot",
        "-monitor", f"unix:{MON},server,nowait",
    ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    # Only a VOID signature (or QEMU gone) retries. No banner yet after two minutes is not void --
    # a slow TCG boot is still a boot -- so the session goes on and its banner check decides.
    void = False
    for _ in range(480):
        log = open(SER, "rb").read().decode("latin1")
        if "AGNOS kernel v" in log:
            break
        if VOID.search(log) or qemu.poll() is not None:
            void = True
            break
        time.sleep(0.25)
    if not void:
        break
    qemu.terminate()
    try:
        qemu.wait(timeout=3)
    except subprocess.TimeoutExpired:
        qemu.kill()
    p(f"  (VOID attempt {attempt}: the firmware never handed off -- retrying with fresh NVRAM)")
else:
    die("the firmware never handed off in 6 attempts (infrastructure, not agnsh)")

# sendkey names for every character this harness types.
KEYS = {" ": "spc", "\n": "ret", "-": "minus", ".": "dot", "/": "slash", "_": "shift-minus",
        ">": "shift-dot", "|": "shift-backslash", "&": "shift-7", "$": "shift-4"}
checks = []            # (name, held)


def check(name, held):
    checks.append((name, bool(held)))
    p(("  ok    " if held else "  FAIL  ") + name)


def records(text):
    """Complete JSON audit records in `text`: one per line, `{"timestamp":` ... `}`."""
    out = []
    for line in text.replace("\r", "").split("\n"):
        line = line.strip()
        i = line.find('{"timestamp":')
        if i >= 0 and line.endswith("}"):
            out.append(line[i:])
    return out


def field(rec, name):
    m = re.search(r'"' + name + r'":"([^"]*)"', rec)
    return m.group(1) if m else None


NL = ["show me all files", "git status", "list running processes"]
# 2.0.0: SAFE / READ_ONLY NL lines RUN (ADR-008). These two have programs on agnos -- kriya's ls, and
# owl, which SHOW_FILE translates to there because agnos has no cat.
NL_LS = "list files in /"
NL_OWL = "show contents of /redir-target.txt"
# 2.0.1: the bareword launcher looks up the first WORD before it judges the line (ADR-007 decision 1).
# Until then it ran is_safe_path over the whole line first, so any sentence with `$`, `(`, `)`, `;` or
# `..` was refused as "run: unsafe path" on agnos and never reached the parser -- the host was right.
NL_META = "what is in $HOME"
SHELL_META = "ls $HOME"                 # a program's line: still refused, now audited (and 126)
REDIR_OK = "echo hello-redirect > /out.txt"
REDIR_AUDIT = "echo pwned > " + AUDIT
REDIR_LINK = "echo pwned > /redir-link"
PIPE_OK = "ls / | wc"
PIPE_MISSING = "nosuchprog | wc"
# 2.0.1 (agnos pipeline-smoke): a pipeline must come back when its consumer is gone. Since agnos 1.57.9
# a pipe write blocks while the ring is full and any read end is open, so a read end the shell keeps --
# or that stage 1 inherited -- leaves the producer blocked for good and the shell reaping it forever.
PIPE_STREAM = "grep . " + CERT + " | wc"               # 185 KB through a 4080-byte ring, every byte
PIPE_EARLY = "grep . " + CERT + " | echo pipeearly"    # the consumer never reads, and exits at once
PIPE_S2FAIL = "grep . " + CERT + " | notelf"           # stage 2 cannot start; stage 1 must still end
# A stage over spawn_path's 127-byte line cap is refused before anything is armed. Until 2.0.1 the cap
# was checked after stage 1's stdout redirect was armed, which left it armed for the NEXT launch.
PIPE_LONG = "echo " + "x" * 130 + " | wc"
ARM_CHECK = "echo armcheck"
SELFKILL = "selfkill"
BG = "sleeper &"
guest = []                               # the in-guest read-back (section 5)

try:
    mon = None
    for _ in range(60):
        try:
            mon = socket.socket(socket.AF_UNIX)
            mon.connect(MON)
            break
        except OSError:
            mon = None
            time.sleep(0.2)
    if mon is None:
        die("no QEMU monitor")
    mon.settimeout(1.0)

    def drain():
        try:
            while True:
                mon.recv(65536)
        except OSError:
            pass

    def ser():
        try:
            return open(SER, "rb").read().decode("latin1")
        except OSError:
            return ""

    def typ(text):
        for ch in text:
            key = KEYS.get(ch, ch)
            if ch.isupper():
                key = "shift-" + ch.lower()
            mon.sendall(("sendkey " + key + "\n").encode())
            time.sleep(0.10)
            drain()

    def run_wait(cmd, marker, timeout=30, settle=1.5):
        """Type `cmd` + Enter; wait for `marker` in the new serial output, then let it settle.
        Returns everything printed since the command was typed."""
        mark = len(ser())
        typ(cmd + "\n")
        deadline = time.time() + timeout
        while time.time() < deadline and marker not in ser()[mark:]:
            time.sleep(0.5)
        time.sleep(settle)
        return ser()[mark:]

    def prompted(seg):
        return "] >" in seg                   # agnsh's `[MODE] >` prompt came back

    booted = False
    for _ in range(480):
        if "agnoshi" in ser():
            booted = True
            break
        time.sleep(0.25)
    check("agnsh reached its banner on agnos", booted)
    if not booted:
        p(ser()[-800:])
        sys.exit(1)
    time.sleep(1.0)
    typ("\n")                                  # absorb the swallowed first keystroke
    time.sleep(1.0)

    # ---- 1. natural language: one parse-time record each ----
    for line in NL:
        seg = run_wait(line, "Risk:")
        check(f"NL {line!r} answered", "Command:" in seg and "Risk:" in seg)
    # ...and since 2.0.0 they run: the report block, then the program's own output.
    seg = run_wait(NL_LS, "] >", timeout=40)
    check("NL exec: `list files in /` runs ls and lists /bin", "Command: ls" in seg and "\nbin" in seg.replace("\r", ""))
    seg = run_wait(NL_OWL, "REDIRECT-TARGET", timeout=40)
    check("NL exec: SHOW_FILE reads through owl on agnos",
          "Command: owl" in seg and "REDIRECT-TARGET-ORIGINAL" in seg)
    seg = run_wait(NL_META, "] >", timeout=40)
    check("a sentence with `$` reaches the NL parser (ADR-007 decision 1)",
          "Intent:" in seg and "unsafe path" not in seg)
    seg = run_wait(SHELL_META, "] >")
    check("a program's line with `$` is refused", "run: unsafe path" in seg)

    # ---- 2. redirection ----
    # Control first: a plain open FOLLOWS the planted symlink, so a later refusal to redirect onto
    # it is AO_NOFOLLOW's doing and not a kernel that cannot resolve symlinks at all.
    seg = run_wait("owl -p /redir-link", "REDIRECT-TARGET")
    check("control: a plain read follows the planted symlink", "REDIRECT-TARGET-ORIGINAL" in seg)
    seg = run_wait(REDIR_OK, "] >")
    check("`cmd > file` ran without an error", "run:" not in seg)
    seg = run_wait("owl -p /out.txt", "hello-redirect")
    check("...and the file holds the command's output", "\nhello-redirect" in seg.replace("\r", ""))

    seg = run_wait(REDIR_AUDIT, "refusing")
    check("`> " + AUDIT + "` is refused", "refusing to redirect over the shell's audit log" in seg)

    seg = run_wait(REDIR_LINK, "] >")
    check("`>` onto a planted symlink is refused", "cannot open redirect target" in seg)
    seg = run_wait("owl -p /redir-target.txt", "REDIRECT-TARGET")
    check("...and the symlink's target is untouched",
          "REDIRECT-TARGET-ORIGINAL" in seg and "\npwned" not in seg.replace("\r", ""))

    # ---- 3. pipelines ----
    seg = run_wait(PIPE_OK, "] >")
    check("`cmd1 | cmd2` ran without an error", "run:" not in seg and prompted(seg))
    seg = run_wait(PIPE_MISSING, "no such command")
    n_missing = seg.count("no such command in pipeline stage 1")
    check("a missing pipeline binary is reported exactly once", n_missing == 1)
    check("...and is not retried down the NL path", "Intent:" not in seg)

    # ---- 3b. pipelines whose consumer is gone (2.0.1) ----
    def must_return(seg, line):
        """A line that never gives the prompt back wedges the session: say which, and stop."""
        if prompted(seg):
            return
        check(f"{line!r} returned to the prompt", False)
        p("  the shell is stuck; serial tail:")
        p(ser()[-600:])
        p(f"agnos-qemu-test: {sum(1 for _, ok in checks if ok)}/{len(checks)} checks held"
          " -- stopped at a wedged shell")
        p("agnos-qemu-test: FAIL")
        sys.exit(1)

    seg = run_wait(PIPE_STREAM, "] >", timeout=90)
    must_return(seg, PIPE_STREAM)
    check(f"a {CERT_NONEMPTY}-byte pipeline streams through the ring (wc counts every byte)",
          str(CERT_NONEMPTY) in seg and "run:" not in seg)
    seg = run_wait(PIPE_EARLY, "] >", timeout=60)
    must_return(seg, PIPE_EARLY)
    check("a pipeline whose consumer never reads returns to the prompt",
          "\npipeearly" in seg.replace("\r", ""))
    seg = run_wait(PIPE_S2FAIL, "] >", timeout=60)
    must_return(seg, PIPE_S2FAIL)
    check("a pipeline whose stage 2 cannot start returns, and says why",
          "failed to launch pipeline stage 2" in seg)
    seg = run_wait(PIPE_LONG, "] >", timeout=60)
    must_return(seg, PIPE_LONG)
    check("a pipeline stage over 127 bytes is refused with a reason", "pipeline stage too long" in seg)
    seg = run_wait(ARM_CHECK, "] >")
    check("...and arms nothing: the next command's output reaches the console",
          "\narmcheck" in seg.replace("\r", ""))

    # ---- 3c. a child killed by a signal (2.0.1) ----
    seg = run_wait(SELFKILL, "] >", timeout=30)
    must_return(seg, SELFKILL)
    check("a SIGKILLed child reports 128 + 9 (ADR-008 § 4), not agnos's 0x100 | 9",
          "run: exit 137" in seg)

    # ---- 4. background jobs, and the job-table cap ----
    launched = 0
    for n in range(1, JOB_CAP + 1):
        seg = run_wait(BG, f"[{n}]", timeout=20)
        launched += f"[{n}]" in seg
    check(f"{JOB_CAP} background jobs launched ([1]..[{JOB_CAP}])", launched == JOB_CAP)
    seg = run_wait(BG, "too many", timeout=20)
    check(f"job {JOB_CAP + 1} is refused", "too many background jobs" in seg)
    check(f"...and never gets a job number", f"[{JOB_CAP + 1}]" not in seg)
    # Release them. Every sleeper prints SLEEPER-DONE as it exits; a ninth one would mean the
    # refused job was spawned anyway -- a stray child agnsh does not track.
    typ("touch /stop\n")
    deadline = time.time() + 180
    while time.time() < deadline and ser().count("] Done") < JOB_CAP:
        time.sleep(1.0)
    time.sleep(5.0)
    out = ser()
    check(f"all {JOB_CAP} jobs were reaped ([n] Done)", out.count("] Done") == JOB_CAP)
    check(f"exactly {JOB_CAP} sleepers ran -- no stray child", out.count("SLEEPER-DONE") == JOB_CAP)

    # ---- 5. the log, read back in the guest ----
    # Wait for agnsh's prompt to come BACK, not for the first record: agnos's console prints the
    # whole log at well under a kilobyte a second, and a line typed while owl is still printing
    # is lost (a first cut of this harness settled 3 s, counted 23 of 37 records, then typed
    # `poweroff` into the running owl -- and the machine never powered off).
    mark = len(ser())
    seg = run_wait("owl -p " + AUDIT, "] >", timeout=120, settle=1.0)
    guest = records(seg)
    p(f"  (owl printed {len(guest)} complete record(s))")

    # ---- 6. poweroff; the disk image is the final word ----
    typ("poweroff\n")
    try:
        qemu.wait(timeout=60)
        powered_off = True
    except subprocess.TimeoutExpired:
        powered_off = False
    check("agnsh `poweroff` ended the VM", powered_off)
finally:
    if qemu.poll() is None:
        qemu.terminate()
        try:
            qemu.wait(timeout=3)
        except subprocess.TimeoutExpired:
            qemu.kill()

part = os.path.join(WORK, "agnos-fs.img")
subprocess.run(["dd", f"if={IMG}", f"of={part}", "bs=1M", "skip=33", "count=67", "status=none"], check=True)
r = subprocess.run(["debugfs", "-R", "cat " + AUDIT, part], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
disk_log = r.stdout.decode("latin1")
open(os.path.join(WORK, "audit-from-disk.log"), "w").write(disk_log)
disk = records(disk_log)
lines = [l for l in disk_log.replace("\r", "").split("\n") if l.strip()]
p(f"  (disk: {len(lines)} line(s), {len(disk)} complete record(s); {WORK}/audit-from-disk.log)")
check("every line on disk is a complete record (no torn overwrite)", lines and len(lines) == len(disk))
# The in-guest read and the disk image must agree: what owl printed is exactly the log's first
# records (the disk then adds owl's own outcome and the poweroff launch).
check(f"the guest's read-back ({len(guest)} records) matches the disk log's first records",
      len(guest) >= 30 and guest == disk[:len(guest)])


def results_for(inp):
    return [field(r, "result") for r in disk if field(r, "input") == inp]


check("disk keeps the three NL records, in order",
      [field(r, "input") for r in disk if field(r, "result") == "proposed"][:3] == NL)
check("NL exec: the ls line is proposed, launched, then executed",
      results_for(NL_LS) == ["proposed", "launched", "executed"])
check("NL exec: the owl line is proposed, launched, then executed",
      results_for(NL_OWL) == ["proposed", "launched", "executed"])
check("`cmd > file`: launched, then executed", results_for(REDIR_OK) == ["launched", "executed"])
check("`> " + AUDIT + "`: denied, nothing launched", results_for(REDIR_AUDIT) == ["denied"])
check("`>` onto a symlink: launched, then error", results_for(REDIR_LINK) == ["launched", "error"])
check("`cmd1 | cmd2`: launched, then executed", results_for(PIPE_OK) == ["launched", "executed"])
check("missing pipeline binary: one error record", results_for(PIPE_MISSING) == ["error"])
check("streaming pipeline: launched, then executed", results_for(PIPE_STREAM) == ["launched", "executed"])
check("early-exit consumer: launched, then executed (stage 2's status)",
      results_for(PIPE_EARLY) == ["launched", "executed"])
check("stage 2 that cannot start: launched, then error", results_for(PIPE_S2FAIL) == ["launched", "error"])
check("over-long pipeline stage: one error record, nothing launched", results_for(PIPE_LONG) == ["error"])
check("the command after it: launched, then executed", results_for(ARM_CHECK) == ["launched", "executed"])
check("the refused program line is on disk as denied", results_for(SHELL_META) == ["denied"])
sk = [r for r in disk if field(r, "input") == SELFKILL]
check("the SIGKILLed child: launched, then failed with exit_code 137",
      [field(r, "result") for r in sk] == ["launched", "failed"] and '"exit_code":137' in sk[-1])
bg = results_for(BG)
check(f"`{BG}`: {JOB_CAP} launched then one denied", bg == ["launched"] * JOB_CAP + ["denied"])
# A reaped job is recorded under its OWN command, not the last line typed (the 1.9.10 asymmetry).
done = [r for r in disk if field(r, "input") == "/bin/sleeper"]
check(f"each reaped job has its own outcome record ({JOB_CAP} x executed)",
      len(done) == JOB_CAP and all(field(r, "result") == "executed" for r in done))
check("the poweroff launch is the last record", disk and field(disk[-1], "input") == "poweroff")

held = sum(1 for _, ok in checks if ok)
p(f"agnos-qemu-test: {held}/{len(checks)} checks held")
if checks and held == len(checks):
    p("agnos-qemu-test: PASS")
    sys.exit(0)
p("agnos-qemu-test: FAIL")
sys.exit(1)
