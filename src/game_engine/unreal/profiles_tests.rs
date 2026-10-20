//! Tests pinning the public profile constants to the versions that pick
//! them.

use super::{profiles::*, Profile, Version};

use std::format;

// `Signature` has no `PartialEq`, so the profiles compare by their debug
// output.
fn same(a: &Profile, b: &Profile) -> bool {
    format!("{a:?}") == format!("{b:?}")
}

#[test]
fn every_version_picks_the_constant_of_its_layout() {
    let versions = [
        (Version::V4_23, UNREAL_4_23_X86_64),
        (Version::V4_24, UNREAL_4_23_X86_64),
        (Version::V4_25, UNREAL_4_25_X86_64),
        (Version::V4_26, UNREAL_4_25_X86_64),
        (Version::V4_27, UNREAL_4_25_X86_64),
        (Version::V5_0, UNREAL_4_25_X86_64),
        (Version::V5_1, UNREAL_4_25_X86_64),
        (Version::V5_2, UNREAL_4_25_X86_64),
        (Version::V5_3, UNREAL_5_3_X86_64),
        (Version::V5_4, UNREAL_5_3_X86_64),
    ];
    for (version, profile) in versions {
        assert!(same(version.profile(), &profile), "{version:?}");
    }
}
