error_chain!{
    foreign_links {
        Io(::std::io::Error);
        Alsa(::alsa::Error);
    }
}
