use http::Request;
use server::Server;

mod http;
mod server; // to find the server module in the server.rs file

fn main() {
    // rust has two types of strings
    // String which is a string, it has dynamic length.
    // And &str is a string slice -> an immutable refrence to a part of a string, meaning it has a fixed length.
    // The compiler will auth convert a String into a &str when a param/var is of type 
    // &str and we are Refrencing a String.
    // let test = String::from("127.0.0.1:8080"); 
    // 10..14 is actually refrencing the byts of the string and not the character
    // in world applications we don't want to slice through strings like that,
    // because some characters takes up more than 1 byte.
    // let string_slice = &test[10..14]; // this is a string slice

    let server = Server::new("127.0.0.1:8080".to_string());
    server.run();
}
