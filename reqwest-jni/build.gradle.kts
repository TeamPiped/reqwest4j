plugins {
    id("fr.stardustenterprises.rust.wrapper")
}

rust {
    release.set(true)
    command.set("cross")

    targets += target("aarch64-unknown-linux-gnu", "libreqwest.so")
    targets += target("x86_64-unknown-linux-gnu", "libreqwest.so")
    targets += target("x86_64-pc-windows-msvc", "libreqwest.dll")
}
