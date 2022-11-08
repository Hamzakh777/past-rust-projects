// enums in memory are represented in numbers in an index way,
// starting from 0 and up.
// each element in an anum is called a 'Variant'.
// A 'Variant' can have a type ex-> Get(String)
pub enum Method {
    GET,
    DELETE,
    POST,
    PUT,
    HEAD,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}
