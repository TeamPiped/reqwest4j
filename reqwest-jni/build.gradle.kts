plugins {
    id("fr.stardustenterprises.rust.wrapper")
}

rust {
    release.set(true)
    command.set("cross")

    targets += target("aarch64-unknown-linux-gnu", "libreqwest.so")
    targets += target("x86_64-unknown-linux-gnu", "libreqwest.so")
    targets += target("aarch64-apple-darwin", "libreqwest.dylib")
    targets += target("x86_64-apple-darwin", "libreqwest.dylib")
    targets += target("x86_64-pc-windows-gnu", "libreqwest.dll")
}
