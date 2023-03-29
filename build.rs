extern crate cmake;
use cmake::Config;

fn main()
{
    let dst = Config::new("lokinet")
        .configure_arg("-DBUILD_STATIC_DEPS=ON")
        .configure_arg("-DSTATIC_LINK=ON")
        .configure_arg("-DWITH_BOOTSTRAP=OFF")
        .configure_arg("-DWITH_EMBEDDED_LOKINET=ON")
        .configure_arg("-DWITH_SYSTEMD=OFF")
        .build();       

    println!("cargo:rustc-link-search=native={}/lib/", dst.display());
    println!("cargo:rustc-link-lib=dylib=lokinet");    
}
