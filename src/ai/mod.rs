mod git;
mod llm;

use clap::Command;
use crate::ai::git::{git_stage_diff, git_stage_filenames};

/// return a clap::Command
pub fn get_command() -> Command {
    let matches = Command::new("ai")
        .about("Using LLM to generate commits");

    matches
}

pub fn handle_command(_: &clap::ArgMatches) {
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

    let filenames = git_stage_filenames();
    if filenames.is_empty() {
        println!("No files added to staging! Did you forget to run `git add` ?");
        return;
    }

    let diff_content = git_stage_diff();

    let commit_text = llm::openai_request(&diff_content).unwrap();

    // 交互式确认是否生成
    if !llm::confirm_commit(&commit_text) {
        println!("用户取消提交");
        return;
    }

    // 提交代码
    git::git_commit(&commit_text);
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
