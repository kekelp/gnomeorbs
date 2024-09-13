use std::fmt;
use std::error::Error;


#[derive(Debug, Clone)]
pub struct NotAFileError;

impl fmt::Display for NotAFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Not a file")
    }
}
impl Error for NotAFileError {}




#[derive(Debug, Clone)]
pub struct NotExecutableError;

impl fmt::Display for NotExecutableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Not an executable file, run \"chmod +x FILE\" first?")
    }
}
impl Error for NotExecutableError {}



#[derive(Debug, Clone)]
pub struct NoHomeDirError;

impl fmt::Display for NoHomeDirError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not find the user's home directory")
    }
}
impl Error for NoHomeDirError {}


#[derive(Debug, Clone)]
pub struct CustomAlreadyExistsError;

impl fmt::Display for CustomAlreadyExistsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, 
        "The target application file already exists. You can use \"-o\" to overwrite it."
        )
    }
}
impl Error for CustomAlreadyExistsError {}



#[derive(Debug, Clone)]
pub struct NonUnicodePathError;

impl fmt::Display for NonUnicodePathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The path to the binary file is not valid Unicode, will not attempt to write it into the application file")
    }
}
impl Error for NonUnicodePathError {}

#[derive(Debug, Clone)]
pub struct NonUnicodeNameError;

impl fmt::Display for NonUnicodeNameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The name of the binary file is not valid Unicode, will not attempt to write it into the application file")
    }
}
impl Error for NonUnicodeNameError {}
