#[cfg(feature = "build-docker-image")]
use anyhow::{bail, Result};
#[cfg(feature = "build-docker-image")]
use std::process::Command;

#[cfg(any(feature = "build-docker-image"))]
fn main() -> Result<()> {
    docker_build()?;
    // ...
    Ok(())
}

#[cfg(not(any(feature = "build-docker-image")))]
fn main() {}

#[cfg(feature = "build-docker-image")]
fn docker_build() -> Result<()> {
    let dockerfile_dir = format!("{}/..", env!("CARGO_MANIFEST_DIR"));
    let dockerfile = format!("{dockerfile_dir}/Dockerfile");
    let image_name = option_env!("CARGO_PKG_NAME").unwrap_or("app");
    let image_label = option_env!("CARGO_PKG_VERSION").unwrap_or("latest");
    let image_tag = format!("{}:{}", image_name, image_label);
    let image_build_arg = format!(
        "BASE_IMAGE_BUILD={}",
        option_env!("BASE_IMAGE_BUILD").unwrap_or("rust:slim-bullseye")
    );
    let image_run_arg = format!(
        "BASE_IMAGE_RUN={}",
        option_env!("BASE_IMAGE_RUN").unwrap_or("gcr.io/distroless/cc")
    );

    // Build the test images in the repository
    let output = Command::new("docker")
        .arg("buildx")
        .arg("build")
        .arg("--file")
        .arg(&dockerfile)
        .arg("--tag")
        .arg(&image_tag)
        .arg("--build-arg")
        .arg(image_build_arg)
        .arg("--build-arg")
        .arg(image_run_arg)
        .arg(dockerfile_dir)
        .output()?;

    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8(output.stderr)?);
        bail!("unable to build {}", &image_tag);
    }
    eprintln!("Built {}", &image_tag);

    println!("cargo:rerun-if-changed={dockerfile}");
    Ok(())
}
