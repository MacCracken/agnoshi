use std::collections::HashMap;
use std::sync::LazyLock;

use super::Interpreter;

static EXPLANATIONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::with_capacity(140);

    // File operations
    m.insert(
        "ls",
        "Lists files and directories in the current or specified directory",
    );
    m.insert(
        "cat",
        "Displays the contents of one or more files to standard output",
    );
    m.insert("cd", "Changes the current working directory");
    m.insert("mkdir", "Creates a new directory");
    m.insert("cp", "Copies files or directories to a destination");
    m.insert("mv", "Moves or renames files and directories");
    m.insert("rm", "Removes files or directories permanently; use with caution as deleted files cannot be easily recovered");
    m.insert(
        "touch",
        "Creates an empty file or updates the timestamp of an existing file",
    );
    m.insert("chmod", "Changes file or directory permissions; incorrect permissions can lock you out of files or expose them to unauthorized access");
    m.insert("chown", "Changes the owner and/or group of a file or directory; requires appropriate privileges and incorrect usage can break file access");
    m.insert(
        "ln",
        "Creates hard or symbolic links to files or directories",
    );
    m.insert(
        "stat",
        "Displays detailed information about a file or filesystem object",
    );
    m.insert(
        "file",
        "Determines the type of a file by examining its contents",
    );
    m.insert("head", "Displays the first lines of a file (default 10)");
    m.insert("tail", "Displays the last lines of a file (default 10)");
    m.insert(
        "less",
        "Displays file contents one screen at a time with backward navigation",
    );
    m.insert("more", "Displays file contents one screen at a time");
    m.insert("wc", "Counts lines, words, and bytes in files");
    m.insert(
        "sort",
        "Sorts lines of text files alphabetically or numerically",
    );
    m.insert(
        "uniq",
        "Filters out adjacent duplicate lines from sorted input",
    );
    m.insert(
        "diff",
        "Compares two files line by line and shows differences",
    );
    m.insert("patch", "Applies a diff patch to files");
    m.insert(
        "realpath",
        "Resolves and prints the absolute path of a file",
    );
    m.insert("readlink", "Prints the target of a symbolic link");
    m.insert(
        "basename",
        "Strips the directory path and returns only the filename",
    );
    m.insert(
        "dirname",
        "Strips the filename and returns only the directory path",
    );

    // Search/filter
    m.insert(
        "grep",
        "Searches for text patterns in files using regular expressions",
    );
    m.insert(
        "find",
        "Finds files and directories matching specified criteria",
    );
    m.insert(
        "locate",
        "Quickly finds files by name using a pre-built database",
    );
    m.insert("which", "Shows the full path of a command executable");
    m.insert(
        "whereis",
        "Locates the binary, source, and manual page for a command",
    );
    m.insert("xargs", "Builds and executes commands from standard input");
    m.insert(
        "awk",
        "Processes and transforms text using pattern-action rules",
    );
    m.insert(
        "sed",
        "Performs text transformations on streams or files using substitution rules",
    );
    m.insert(
        "cut",
        "Extracts specific columns or fields from each line of input",
    );
    m.insert(
        "tr",
        "Translates, deletes, or squeezes characters from standard input",
    );

    // Archive/compression
    m.insert(
        "tar",
        "Creates or extracts archive files, optionally with compression",
    );
    m.insert(
        "zip",
        "Creates compressed zip archives from files and directories",
    );
    m.insert("unzip", "Extracts files from zip archives");
    m.insert("gzip", "Compresses files using the gzip algorithm");
    m.insert("gunzip", "Decompresses gzip-compressed files");
    m.insert("bzip2", "Compresses files using the bzip2 algorithm");
    m.insert("xz", "Compresses files using the xz/LZMA algorithm");
    m.insert(
        "zstd",
        "Compresses or decompresses files using the Zstandard algorithm",
    );

    // Process management
    m.insert("ps", "Lists currently running processes");
    m.insert(
        "top",
        "Displays real-time system resource usage and running processes",
    );
    m.insert(
        "htop",
        "Displays an interactive, color-coded view of running processes and system resources",
    );
    m.insert("kill", "Sends a signal to a process, typically to terminate it; ensure you target the correct PID to avoid stopping critical processes");
    m.insert("killall", "Sends a signal to all processes matching a name; use carefully as it affects every matching process");
    m.insert("pkill", "Sends a signal to processes matching a pattern; use carefully as it can match more processes than intended");
    m.insert("nice", "Runs a command with a modified scheduling priority");
    m.insert(
        "renice",
        "Changes the scheduling priority of a running process",
    );
    m.insert(
        "nohup",
        "Runs a command that persists after the terminal session ends",
    );
    m.insert("bg", "Resumes a suspended job in the background");
    m.insert("fg", "Brings a background job to the foreground");
    m.insert("jobs", "Lists active jobs in the current shell session");
    m.insert(
        "wait",
        "Waits for background processes to finish before continuing",
    );

    // System info
    m.insert("df", "Shows disk space usage for mounted filesystems");
    m.insert("du", "Shows disk space usage for files and directories");
    m.insert(
        "free",
        "Displays the amount of free and used memory in the system",
    );
    m.insert(
        "uname",
        "Prints system information such as kernel name and version",
    );
    m.insert(
        "uptime",
        "Shows how long the system has been running and the load average",
    );
    m.insert("hostname", "Displays or sets the system hostname");
    m.insert("lsb_release", "Displays Linux distribution information");
    m.insert("lscpu", "Displays detailed CPU architecture information");
    m.insert(
        "lsblk",
        "Lists information about block devices such as disks and partitions",
    );
    m.insert("lsusb", "Lists USB devices connected to the system");
    m.insert("lspci", "Lists PCI devices connected to the system");
    m.insert(
        "dmesg",
        "Displays kernel ring buffer messages, useful for hardware and driver diagnostics",
    );
    m.insert(
        "mount",
        "Mounts a filesystem or displays currently mounted filesystems",
    );
    m.insert("umount", "Unmounts a mounted filesystem");

    // Network
    m.insert(
        "ping",
        "Sends ICMP echo requests to test network connectivity to a host",
    );
    m.insert(
        "curl",
        "Transfers data from or to a server using various protocols",
    );
    m.insert(
        "wget",
        "Downloads files from the web via HTTP, HTTPS, or FTP",
    );
    m.insert("ssh", "Opens a secure shell connection to a remote host");
    m.insert("scp", "Copies files securely between hosts over SSH");
    m.insert(
        "rsync",
        "Synchronizes files and directories between locations efficiently",
    );
    m.insert(
        "ip",
        "Configures and displays network interfaces, routes, and addresses",
    );
    m.insert(
        "ifconfig",
        "Displays or configures network interface parameters",
    );
    m.insert(
        "netstat",
        "Displays network connections, routing tables, and interface statistics",
    );
    m.insert("ss", "Displays socket statistics and network connections");
    m.insert(
        "dig",
        "Performs DNS lookups and displays detailed query results",
    );
    m.insert(
        "nslookup",
        "Queries DNS servers to resolve hostnames to IP addresses",
    );
    m.insert(
        "traceroute",
        "Traces the network path packets take to reach a destination host",
    );
    m.insert(
        "nc",
        "Reads and writes data across network connections (netcat)",
    );

    // User/permission
    m.insert("sudo", "Executes a command with superuser (root) privileges; be careful as root access can modify or destroy any file on the system");
    m.insert("su", "Switches the current user to another user account");
    m.insert("whoami", "Prints the current effective username");
    m.insert(
        "id",
        "Displays the current user's UID, GID, and group memberships",
    );
    m.insert("groups", "Displays the groups a user belongs to");
    m.insert("passwd", "Changes a user's password");
    m.insert("useradd", "Creates a new user account on the system");
    m.insert("userdel", "Deletes a user account from the system");
    m.insert("usermod", "Modifies an existing user account's properties");
    m.insert("groupadd", "Creates a new group on the system");

    // Package management
    m.insert(
        "apt",
        "Manages packages on Debian-based systems (install, remove, update)",
    );
    m.insert(
        "apt-get",
        "Manages packages on Debian-based systems (lower-level interface)",
    );
    m.insert(
        "dpkg",
        "Installs, removes, and inspects individual .deb packages",
    );
    m.insert("ark", "Manages packages using the Ark package manager");
    m.insert("pip", "Installs and manages Python packages from PyPI");
    m.insert(
        "cargo",
        "Builds, tests, and manages Rust projects and dependencies",
    );
    m.insert(
        "npm",
        "Installs and manages Node.js packages from the npm registry",
    );

    // Service/system
    m.insert(
        "systemctl",
        "Controls and inspects systemd services and units",
    );
    m.insert("journalctl", "Views and queries systemd journal logs");
    m.insert(
        "crontab",
        "Schedules commands to run automatically at specified times",
    );
    m.insert(
        "at",
        "Schedules a one-time command to run at a specified time",
    );
    m.insert(
        "reboot",
        "Restarts the system immediately; ensure all work is saved before running",
    );
    m.insert(
        "shutdown",
        "Shuts down or schedules a shutdown of the system; ensure all work is saved before running",
    );
    m.insert(
        "poweroff",
        "Powers off the system immediately; ensure all work is saved before running",
    );
    m.insert(
        "loginctl",
        "Controls and inspects the systemd login manager and user sessions",
    );

    // Development
    m.insert("git", "Manages source code version control repositories");
    m.insert(
        "make",
        "Builds projects by executing rules defined in a Makefile",
    );
    m.insert("gcc", "Compiles C and C++ source code into executables");
    m.insert(
        "python",
        "Runs Python scripts or starts an interactive Python interpreter",
    );
    m.insert(
        "python3",
        "Runs Python 3 scripts or starts an interactive Python 3 interpreter",
    );
    m.insert(
        "node",
        "Runs JavaScript code or starts an interactive Node.js REPL",
    );
    m.insert("rustc", "Compiles Rust source code into executables");
    m.insert(
        "docker",
        "Manages containers, images, and container-based applications",
    );
    m.insert(
        "kubectl",
        "Controls and manages Kubernetes clusters and workloads",
    );

    // Firewall
    m.insert(
        "ufw",
        "Manages the Uncomplicated Firewall — allows, denies, and lists rules for network traffic",
    );
    m.insert(
        "nft",
        "Manages nftables firewall rules for packet filtering and classification",
    );
    m.insert(
        "iptables",
        "Manages IPv4 packet filter rules in the Linux kernel firewall",
    );
    m.insert(
        "ip6tables",
        "Manages IPv6 packet filter rules in the Linux kernel firewall",
    );
    m.insert("groupdel", "Deletes a group from the system");

    // Misc
    m.insert("echo", "Prints text or variable values to standard output");
    m.insert(
        "env",
        "Displays or modifies the current environment variables",
    );
    m.insert(
        "export",
        "Sets an environment variable available to child processes",
    );
    m.insert(
        "source",
        "Executes commands from a file in the current shell environment",
    );
    m.insert(
        "alias",
        "Creates a shortcut name for a command or command sequence",
    );
    m.insert(
        "history",
        "Displays the list of previously executed commands",
    );
    m.insert("clear", "Clears the terminal screen");
    m.insert("date", "Displays or sets the current date and time");
    m.insert(
        "cal",
        "Displays a calendar for the current or specified month",
    );
    m.insert("man", "Displays the manual page for a command");
    m.insert(
        "help",
        "Displays help information for shell built-in commands",
    );
    m.insert(
        "type",
        "Indicates how a command name would be interpreted by the shell",
    );
    m.insert(
        "tee",
        "Reads from standard input and writes to both standard output and files",
    );
    m.insert(
        "xclip",
        "Copies or pastes text to and from the X11 clipboard",
    );

    m
});

impl Interpreter {
    /// Get explanation of what a command does
    #[must_use]
    pub fn explain(&self, command: &str, _args: &[String]) -> String {
        let cmd = command.to_lowercase();
        EXPLANATIONS
            .get(cmd.as_str())
            .map(|s| (*s).to_string())
            .unwrap_or_else(|| format!("Executes the {} command", cmd))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_known_command_ls() {
        let interp = make_interpreter();
        let result = interp.explain("ls", &[]);
        assert!(
            result.contains("files and directories"),
            "ls explanation should mention files and directories"
        );
        assert!(
            !result.starts_with("Executes the"),
            "ls should not return the generic fallback"
        );
    }

    #[test]
    fn test_known_command_rm() {
        let interp = make_interpreter();
        let result = interp.explain("rm", &[]);
        assert!(
            result.contains("caution"),
            "rm explanation should include a safety note"
        );
    }

    #[test]
    fn test_known_command_grep() {
        let interp = make_interpreter();
        let result = interp.explain("grep", &[]);
        assert!(
            result.contains("pattern"),
            "grep explanation should mention patterns"
        );
    }

    #[test]
    fn test_known_command_sudo() {
        let interp = make_interpreter();
        let result = interp.explain("sudo", &[]);
        assert!(
            result.contains("superuser") || result.contains("root"),
            "sudo explanation should mention elevated privileges"
        );
        assert!(
            result.contains("careful") || result.contains("caution"),
            "sudo explanation should include a safety note"
        );
    }

    #[test]
    fn test_known_command_chmod() {
        let interp = make_interpreter();
        let result = interp.explain("chmod", &[]);
        assert!(
            result.contains("permission"),
            "chmod explanation should mention permissions"
        );
    }

    #[test]
    fn test_known_command_kill() {
        let interp = make_interpreter();
        let result = interp.explain("kill", &[]);
        assert!(
            result.contains("signal") || result.contains("terminate"),
            "kill explanation should mention signals or termination"
        );
    }

    #[test]
    fn test_known_command_tar() {
        let interp = make_interpreter();
        let result = interp.explain("tar", &[]);
        assert!(
            result.contains("archive"),
            "tar explanation should mention archives"
        );
    }

    #[test]
    fn test_known_command_git() {
        let interp = make_interpreter();
        let result = interp.explain("git", &[]);
        assert!(
            result.contains("version control"),
            "git explanation should mention version control"
        );
    }

    #[test]
    fn test_known_command_docker() {
        let interp = make_interpreter();
        let result = interp.explain("docker", &[]);
        assert!(
            result.contains("container"),
            "docker explanation should mention containers"
        );
    }

    #[test]
    fn test_known_command_reboot() {
        let interp = make_interpreter();
        let result = interp.explain("reboot", &[]);
        assert!(
            result.contains("Restart") || result.contains("restart") || result.contains("Restarts"),
            "reboot explanation should mention restarting"
        );
        assert!(
            result.contains("saved") || result.contains("ensure"),
            "reboot explanation should include a safety note"
        );
    }

    #[test]
    fn test_known_command_curl() {
        let interp = make_interpreter();
        let result = interp.explain("curl", &[]);
        assert!(
            result.contains("Transfer") || result.contains("transfer"),
            "curl explanation should mention data transfer"
        );
    }

    #[test]
    fn test_unknown_command_returns_fallback() {
        let interp = make_interpreter();
        let result = interp.explain("nonexistentcommand123", &[]);
        assert_eq!(result, "Executes the nonexistentcommand123 command");
    }

    #[test]
    fn test_unknown_command_fallback_format() {
        let interp = make_interpreter();
        let result = interp.explain("zzzyyyxxx", &[]);
        assert!(
            result.starts_with("Executes the"),
            "Unknown commands should return the generic fallback"
        );
        assert!(
            result.contains("zzzyyyxxx"),
            "Fallback should include the command name"
        );
    }

    #[test]
    fn test_case_insensitivity() {
        let interp = make_interpreter();
        let upper = interp.explain("LS", &[]);
        let lower = interp.explain("ls", &[]);
        assert_eq!(upper, lower, "Command lookup should be case-insensitive");
    }
}
