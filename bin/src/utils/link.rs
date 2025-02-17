use crate::error::Error;

#[allow(clippy::module_name_repetitions)]
#[cfg(windows)]
/// Create a symbolic link
///
/// # Errors
/// - [`Error::Link`] if the link could not be created
/// - [`std::io::Error`] if the link command could not be executed
pub fn create_link(link: &str, target: &str) -> Result<(), Error> {
    use std::process::Command;
    trace!("link {:?} => {:?}", link, target);

    // attempt junction
    let mut out = Command::new("cmd")
        .arg("/C")
        .arg("mklink")
        .arg("/J")
        .arg(link)
        .arg(target)
        .output()?;

    if !out.status.success() {
        // fall-back to directory symbolic link
        out = Command::new("cmd")
            .arg("/C")
            .arg("mklink")
            .arg("/D")
            .arg(link)
            .arg(target)
            .output()?;

        if !out.status.success() {
            return Err(Error::Link(
                String::from_utf8_lossy(&out.stderr).to_string(),
            ));
        }
    }
    Ok(())
}

#[allow(clippy::module_name_repetitions)]
#[cfg(not(windows))]
/// Create a symbolic link
///
/// # Errors
/// - [`std::io::Error`] if the link could not be created
pub fn create_link(link: &str, target: &str) -> Result<(), Error> {
    let target = target.replace('\\', "/");
    trace!("link {:?} => {:?}", link, target);
    std::os::unix::fs::symlink(target, link)?;
    Ok(())
}
