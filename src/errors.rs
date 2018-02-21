error_chain!{
    foreign_links {
        Hyper(::hyper::Error);
        Http(::http::Error);
    }
}
