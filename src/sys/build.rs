
extern crate pnacl_build_helper as helper;
extern crate pkg_config as pkg;

pub fn main() {
    helper::set_pkg_config_envs();
    helper::print_lib_paths();

    pkg::Config::new().statik(true).find("vpx").unwrap();
}
