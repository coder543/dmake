extern crate getopts;
extern crate tini;

use getopts::Options;
use tini::Ini;

use std::process::{exit, Command};
use std::env::{args, current_dir, set_current_dir};
use std::path::Path;

fn main() {
    let mut args = args();
    let program = args.next().unwrap();

    let mut opts = Options::new();
    opts.optflag("", "deploy", "deploy to Docker Hub");
    opts.optflag("r", "run", "run image locally after it is built");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(args) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [options] [mode]", program);
        println!("{}", opts.usage(&brief));
        return;
    }

    if matches.opt_present("deploy") && matches.opt_present("r") {
        println!("Running locally and deploying should not be done together");
        return;
    }

    if matches.opt_present("deploy") {
        println!("Sorry, automatic deployment isn't supported yet");
        return;
    }

    let mode = if matches.free.is_empty() {
        "debug".to_string()
    } else {
        matches.free[0].clone()
    };

    // search upwards until we find Dmake.ini
    while !Path::new("Dmake.ini").exists() {
        let cwd = current_dir().unwrap();
        let parent_dir = cwd.parent().unwrap_or_else(|| {
            println!("Dmake.ini does not exist in this directory or any parent directory!");
            exit(1);
        });
        set_current_dir(parent_dir).unwrap();
    }

    let project = Ini::from_file("Dmake.ini").unwrap();

    let image_name: String = project.get("image", "name").unwrap();

    println!("---- BUILDING {}:latest\n", image_name);
    let docker_build = Command::new("docker")
        .arg("build")
        .arg("--build-arg")
        .arg(format!("image_name={}", image_name))
        .arg("-t")
        .arg(format!("{}:latest", image_name))
        .arg("--file")
        .arg(format!("Dockerfile-{}", mode))
        .arg(".")
        .status()
        .unwrap()
        .success();

    assert!(docker_build);

    if matches.opt_present("r") {
        println!("\n---- RUNNING {}:latest\n", image_name);
        let port_num: Option<String> = project.get("image", "port");

        match port_num {
            None => {
                Command::new("docker")
                    .arg("run")
                    .arg("-it")
                    .arg("--rm")
                    .arg(format!("{}:latest", image_name))
                    .spawn()
                    .unwrap();
            }
            Some(port) => {
                Command::new("docker")
                    .arg("run")
                    .arg("-it")
                    .arg("--rm")
                    .arg("-p")
                    .arg(format!("{}:{}", port, port))
                    .arg(format!("{}:latest", image_name))
                    .spawn()
                    .unwrap();
            }
        }
    }
}
