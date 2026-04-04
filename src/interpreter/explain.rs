use super::Interpreter;

impl Interpreter {
    /// Get explanation of what a command does
    #[must_use]
    pub fn explain(&self, command: &str, _args: &[String]) -> String {
        let cmd = command.to_lowercase();

        match cmd.as_str() {
            // File operations
            "ls" => "Lists files and directories in the current or specified directory".to_string(),
            "cat" => "Displays the contents of one or more files to standard output".to_string(),
            "cd" => "Changes the current working directory".to_string(),
            "mkdir" => "Creates a new directory".to_string(),
            "cp" => "Copies files or directories to a destination".to_string(),
            "mv" => "Moves or renames files and directories".to_string(),
            "rm" => "Removes files or directories permanently; use with caution as deleted files cannot be easily recovered".to_string(),
            "touch" => "Creates an empty file or updates the timestamp of an existing file".to_string(),
            "chmod" => "Changes file or directory permissions; incorrect permissions can lock you out of files or expose them to unauthorized access".to_string(),
            "chown" => "Changes the owner and/or group of a file or directory; requires appropriate privileges and incorrect usage can break file access".to_string(),
            "ln" => "Creates hard or symbolic links to files or directories".to_string(),
            "stat" => "Displays detailed information about a file or filesystem object".to_string(),
            "file" => "Determines the type of a file by examining its contents".to_string(),
            "head" => "Displays the first lines of a file (default 10)".to_string(),
            "tail" => "Displays the last lines of a file (default 10)".to_string(),
            "less" => "Displays file contents one screen at a time with backward navigation".to_string(),
            "more" => "Displays file contents one screen at a time".to_string(),
            "wc" => "Counts lines, words, and bytes in files".to_string(),
            "sort" => "Sorts lines of text files alphabetically or numerically".to_string(),
            "uniq" => "Filters out adjacent duplicate lines from sorted input".to_string(),
            "diff" => "Compares two files line by line and shows differences".to_string(),
            "patch" => "Applies a diff patch to files".to_string(),
            "realpath" => "Resolves and prints the absolute path of a file".to_string(),
            "readlink" => "Prints the target of a symbolic link".to_string(),
            "basename" => "Strips the directory path and returns only the filename".to_string(),
            "dirname" => "Strips the filename and returns only the directory path".to_string(),

            // Search/filter
            "grep" => "Searches for text patterns in files using regular expressions".to_string(),
            "find" => "Finds files and directories matching specified criteria".to_string(),
            "locate" => "Quickly finds files by name using a pre-built database".to_string(),
            "which" => "Shows the full path of a command executable".to_string(),
            "whereis" => "Locates the binary, source, and manual page for a command".to_string(),
            "xargs" => "Builds and executes commands from standard input".to_string(),
            "awk" => "Processes and transforms text using pattern-action rules".to_string(),
            "sed" => "Performs text transformations on streams or files using substitution rules".to_string(),
            "cut" => "Extracts specific columns or fields from each line of input".to_string(),
            "tr" => "Translates, deletes, or squeezes characters from standard input".to_string(),

            // Archive/compression
            "tar" => "Creates or extracts archive files, optionally with compression".to_string(),
            "zip" => "Creates compressed zip archives from files and directories".to_string(),
            "unzip" => "Extracts files from zip archives".to_string(),
            "gzip" => "Compresses files using the gzip algorithm".to_string(),
            "gunzip" => "Decompresses gzip-compressed files".to_string(),
            "bzip2" => "Compresses files using the bzip2 algorithm".to_string(),
            "xz" => "Compresses files using the xz/LZMA algorithm".to_string(),
            "zstd" => "Compresses or decompresses files using the Zstandard algorithm".to_string(),

            // Process management
            "ps" => "Lists currently running processes".to_string(),
            "top" => "Displays real-time system resource usage and running processes".to_string(),
            "htop" => "Displays an interactive, color-coded view of running processes and system resources".to_string(),
            "kill" => "Sends a signal to a process, typically to terminate it; ensure you target the correct PID to avoid stopping critical processes".to_string(),
            "killall" => "Sends a signal to all processes matching a name; use carefully as it affects every matching process".to_string(),
            "pkill" => "Sends a signal to processes matching a pattern; use carefully as it can match more processes than intended".to_string(),
            "nice" => "Runs a command with a modified scheduling priority".to_string(),
            "renice" => "Changes the scheduling priority of a running process".to_string(),
            "nohup" => "Runs a command that persists after the terminal session ends".to_string(),
            "bg" => "Resumes a suspended job in the background".to_string(),
            "fg" => "Brings a background job to the foreground".to_string(),
            "jobs" => "Lists active jobs in the current shell session".to_string(),
            "wait" => "Waits for background processes to finish before continuing".to_string(),

            // System info
            "df" => "Shows disk space usage for mounted filesystems".to_string(),
            "du" => "Shows disk space usage for files and directories".to_string(),
            "free" => "Displays the amount of free and used memory in the system".to_string(),
            "uname" => "Prints system information such as kernel name and version".to_string(),
            "uptime" => "Shows how long the system has been running and the load average".to_string(),
            "hostname" => "Displays or sets the system hostname".to_string(),
            "lsb_release" => "Displays Linux distribution information".to_string(),
            "lscpu" => "Displays detailed CPU architecture information".to_string(),
            "lsblk" => "Lists information about block devices such as disks and partitions".to_string(),
            "lsusb" => "Lists USB devices connected to the system".to_string(),
            "lspci" => "Lists PCI devices connected to the system".to_string(),
            "dmesg" => "Displays kernel ring buffer messages, useful for hardware and driver diagnostics".to_string(),
            "mount" => "Mounts a filesystem or displays currently mounted filesystems".to_string(),
            "umount" => "Unmounts a mounted filesystem".to_string(),

            // Network
            "ping" => "Sends ICMP echo requests to test network connectivity to a host".to_string(),
            "curl" => "Transfers data from or to a server using various protocols".to_string(),
            "wget" => "Downloads files from the web via HTTP, HTTPS, or FTP".to_string(),
            "ssh" => "Opens a secure shell connection to a remote host".to_string(),
            "scp" => "Copies files securely between hosts over SSH".to_string(),
            "rsync" => "Synchronizes files and directories between locations efficiently".to_string(),
            "ip" => "Configures and displays network interfaces, routes, and addresses".to_string(),
            "ifconfig" => "Displays or configures network interface parameters".to_string(),
            "netstat" => "Displays network connections, routing tables, and interface statistics".to_string(),
            "ss" => "Displays socket statistics and network connections".to_string(),
            "dig" => "Performs DNS lookups and displays detailed query results".to_string(),
            "nslookup" => "Queries DNS servers to resolve hostnames to IP addresses".to_string(),
            "traceroute" => "Traces the network path packets take to reach a destination host".to_string(),
            "nc" => "Reads and writes data across network connections (netcat)".to_string(),

            // User/permission
            "sudo" => "Executes a command with superuser (root) privileges; be careful as root access can modify or destroy any file on the system".to_string(),
            "su" => "Switches the current user to another user account".to_string(),
            "whoami" => "Prints the current effective username".to_string(),
            "id" => "Displays the current user's UID, GID, and group memberships".to_string(),
            "groups" => "Displays the groups a user belongs to".to_string(),
            "passwd" => "Changes a user's password".to_string(),
            "useradd" => "Creates a new user account on the system".to_string(),
            "userdel" => "Deletes a user account from the system".to_string(),
            "usermod" => "Modifies an existing user account's properties".to_string(),
            "groupadd" => "Creates a new group on the system".to_string(),

            // Package management
            "apt" => "Manages packages on Debian-based systems (install, remove, update)".to_string(),
            "apt-get" => "Manages packages on Debian-based systems (lower-level interface)".to_string(),
            "dpkg" => "Installs, removes, and inspects individual .deb packages".to_string(),
            "ark" => "Manages packages using the Ark package manager".to_string(),
            "pip" => "Installs and manages Python packages from PyPI".to_string(),
            "cargo" => "Builds, tests, and manages Rust projects and dependencies".to_string(),
            "npm" => "Installs and manages Node.js packages from the npm registry".to_string(),

            // Service/system
            "systemctl" => "Controls and inspects systemd services and units".to_string(),
            "journalctl" => "Views and queries systemd journal logs".to_string(),
            "crontab" => "Schedules commands to run automatically at specified times".to_string(),
            "at" => "Schedules a one-time command to run at a specified time".to_string(),
            "reboot" => "Restarts the system immediately; ensure all work is saved before running".to_string(),
            "shutdown" => "Shuts down or schedules a shutdown of the system; ensure all work is saved before running".to_string(),
            "poweroff" => "Powers off the system immediately; ensure all work is saved before running".to_string(),
            "loginctl" => "Controls and inspects the systemd login manager and user sessions".to_string(),

            // Development
            "git" => "Manages source code version control repositories".to_string(),
            "make" => "Builds projects by executing rules defined in a Makefile".to_string(),
            "gcc" => "Compiles C and C++ source code into executables".to_string(),
            "python" => "Runs Python scripts or starts an interactive Python interpreter".to_string(),
            "python3" => "Runs Python 3 scripts or starts an interactive Python 3 interpreter".to_string(),
            "node" => "Runs JavaScript code or starts an interactive Node.js REPL".to_string(),
            "rustc" => "Compiles Rust source code into executables".to_string(),
            "docker" => "Manages containers, images, and container-based applications".to_string(),
            "kubectl" => "Controls and manages Kubernetes clusters and workloads".to_string(),

            // Misc
            "echo" => "Prints text or variable values to standard output".to_string(),
            "env" => "Displays or modifies the current environment variables".to_string(),
            "export" => "Sets an environment variable available to child processes".to_string(),
            "source" => "Executes commands from a file in the current shell environment".to_string(),
            "alias" => "Creates a shortcut name for a command or command sequence".to_string(),
            "history" => "Displays the list of previously executed commands".to_string(),
            "clear" => "Clears the terminal screen".to_string(),
            "date" => "Displays or sets the current date and time".to_string(),
            "cal" => "Displays a calendar for the current or specified month".to_string(),
            "man" => "Displays the manual page for a command".to_string(),
            "help" => "Displays help information for shell built-in commands".to_string(),
            "type" => "Indicates how a command name would be interpreted by the shell".to_string(),
            "tee" => "Reads from standard input and writes to both standard output and files".to_string(),
            "xclip" => "Copies or pastes text to and from the X11 clipboard".to_string(),

            _ => format!("Executes the {} command", cmd),
        }
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
