pub mod campuslink {
    pub mod ncms {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/campuslink.ncms.v1.rs"));
        }
    }
}
