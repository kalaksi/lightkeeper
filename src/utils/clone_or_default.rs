use std::{ default::Default, clone::Clone };

trait CloneOrDefault<T> {
    fn clone_or_default(&self) -> T;
}

/* 
TODO: more generic implementation:

impl<T> CloneOrDefault<T> for Option<&T> where T: Default + Clone {
    fn clone_or_default(&self) -> T {
        match self {
            Some(T) => ,
            None => T::default(),
        }
    }
}
*/

/*
impl<String> CloneOrDefault<String> for Option<&String> {
    fn clone_or_default(&self) -> String {
        match self {
            Some(string) => string.clone(),
            None => String::new(),
        }
    }
}
*/