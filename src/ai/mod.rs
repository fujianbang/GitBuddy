mod git;

use clap::Command;

/// return a clap::Command
pub fn get_command() -> Command {
    let matches = Command::new("ai")
        .about("Using LLM to generate commits");

    matches
}

pub fn handle_command(matches: &clap::ArgMatches) {
    println!("{:?}", matches);

    // 检查当前目录是否为git目录
    if !is_git_directory() {
        println!("当前目录不是git目录");
        return;
    }

    // 检查是否安装git
    if !is_git_installed() {
        println!("请安装git");
        return;
    }

}

fn is_git_directory() -> bool {
    // 获取当前目录
    let current_dir = std::env::current_dir().unwrap();

    // 检查当前目录是否为git目录
    return if current_dir.join(".git").exists() {
        true
    } else {
        false
    };
}

fn is_git_installed() -> bool {
    // 检查是否安装git
    return if std::process::Command::new("git").arg("--version").output().is_ok() {
        true
    } else {
        false
    };
}
