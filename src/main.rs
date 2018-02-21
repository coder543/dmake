extern crate getopts;
extern crate tini;

use getopts::Options;
use tini::Ini;

use std::process::{exit, Command};
use std::env::{args, current_dir, set_current_dir};
use std::fs::{create_dir_all, remove_dir_all};
use std::path::Path;

fn main() {
    let mut args = args();
    let program = args.next().unwrap();

    let mut opts = Options::new();
    opts.optflag("", "deploy", "deploy to Docker Hub");
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

    let docker_build = Command::new("docker")
        .current_dir("build")
        .arg("build")
        .arg("-t")
        .arg(format!("{}-build", image_name))
        .arg("--file")
        .arg(format!("Dockerfile-{}", mode))
        .arg(".")
        .status()
        .unwrap()
        .success();

    assert!(docker_build);

    let volume = &current_dir().unwrap().join("deploy").join(&image_name);
    if let Err(err) = remove_dir_all(volume) {
        println!("WARNING: {:?}", err);
    }
    if let Err(err) = create_dir_all(volume) {
        println!("ERROR: Could not create directory to hold build artifacts.");
        println!("Reason: {:?}", err);
        exit(2);
    }

    let extract_build = Command::new("docker")
        .arg("run")
        .arg("-it")
        .arg("--rm")
        .arg("-v")
        .arg(format!("{}:/opt/{}", volume.display(), image_name))
        .arg(format!("{}-build:latest", image_name))
        .status()
        .unwrap()
        .success();

    assert!(extract_build);

    let build_deployment_image = Command::new("docker")
        .current_dir("deploy")
        .arg("build")
        .arg("-t")
        .arg(&image_name)
        .arg(".")
        .status()
        .unwrap()
        .success();

    assert!(build_deployment_image);
}
