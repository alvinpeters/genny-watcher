use std::fmt::{Display, Formatter};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Clone, Copy, Archive, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[repr(u8)]
pub(crate) enum ArkMap {
    Island = 0,
    ScorchedEarth = 1,
    Center = 2,
    Aberration = 3,
}

impl Display for ArkMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ArkMap::Island => write!(f, "The Island"),
            ArkMap::ScorchedEarth => write!(f, "Scorched Earth"),
            ArkMap::Center => write!(f, "The Center"),
            ArkMap::Aberration => write!(f, "Aberration"),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ArkMapScale {
    pub(crate) latitude_origin: i32,
    pub(crate) longitude_origin: i32,
    pub(crate) scale: f32,
}

enum WithinRange {
    Inside,
    Outside,
    /// Within x, y, but z is missing for either
    Maybe
}

#[derive(Copy, Clone, Archive, Serialize, Deserialize)]
pub(crate) struct UE4Coordinates {
    x: i32,
    y: i32,
    z: Option<i32>,
    map: ArkMap,
}

#[derive(Copy, Clone, Archive, Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct ArkCoordinates {
    latitude: f32,
    longitude: f32,
    map: ArkMap,
}

impl UE4Coordinates {
    pub(crate) fn within_range(&self, other: &Self, range: i32) -> WithinRange {
        if self.map != other.map {
            // They are not even on the same map.
            return WithinRange::Outside;
        }

        fn within_range_without_z(this: &UE4Coordinates, other: &UE4Coordinates, range: i32) -> WithinRange {
            let x_res = (other.x - this.x).pow(2);
            let y_res = (other.y - this.y).pow(2);

            let sum = x_res + y_res;
            let result = (sum as f32).sqrt().floor() as i32;

            if result <= range {
                WithinRange::Maybe
            } else {
                WithinRange::Outside
            }
        }
        let Some(self_z) = self.z else {
            return within_range_without_z(self, other, range);
        };
        let Some(other_z) = other.z else {
            return within_range_without_z(self, other, range);
        };

        let x_res = (other.x - self.x).pow(2);
        let y_res = (other.y - self.y).pow(2);
        let z_res = (other_z - self_z).pow(2);

        let sum = x_res + y_res + z_res;
        let result = (sum as f32).sqrt().floor() as i32;

        if result <= range {
            WithinRange::Inside
        } else {
            WithinRange::Outside
        }
    }
}

impl ArkCoordinates {
    /// Returns ARK Coordinates that are rounded to one decimal place, just like in ARK.
    pub(crate) fn rounded(&self) -> Self {
        Self {
            latitude: (self.latitude * 10.0).round() / 10.0,
            longitude: (self.longitude * 10.0).round() / 10.0,
            map: self.map,
        }
    }
}

impl ArkMap {
    /// Provides the origin and scale values used by coordinate calculations.
    ///
    /// How the values are gathered:
    ///  1. Go to a map in singleplayer
    ///  2. 'gcm', jump to hover, and then 'tpcoords 0 0 100'
    ///  3. 'ccc'
    pub(crate) const fn get_scale(&self) -> ArkMapScale {
        match self {
            ArkMap::Island => ArkMapScale {
                latitude_origin: -342901,
                longitude_origin: -342900,
                scale: 6858.0,
            },
            ArkMap::ScorchedEarth => ArkMapScale {
                latitude_origin: -393650,
                longitude_origin: -393650,
                scale: 7874.0,
            },
            ArkMap::Center => ArkMapScale {
                latitude_origin: -337215,
                longitude_origin: -524364,
                scale: 10374.04,
            },
            ArkMap::Aberration => ArkMapScale {
                latitude_origin: -400000,
                longitude_origin: -400000,
                scale: 8000.0,
            },
        }
    }
}

impl From<ArkCoordinates> for UE4Coordinates {
    fn from(ark_coords: ArkCoordinates) -> Self {
        let map = ark_coords.map;
        let map_scale = map.get_scale();

        let x = (ark_coords.longitude * map_scale.scale).round() as i32 + map_scale.longitude_origin;
        let y = (ark_coords.latitude * map_scale.scale).round() as i32 + map_scale.latitude_origin;

        Self {
            x,
            y,
            z: None,
            map,
        }

    }
}

impl From<UE4Coordinates> for ArkCoordinates {
    /// Converting into ARK coordinates will drop the height / y coordinates, if exists.
    fn from(ue4_coords: UE4Coordinates) -> Self {
        let map = ue4_coords.map;
        let map_scale = map.get_scale();

        let latitude = (ue4_coords.y - map_scale.latitude_origin) as f32 / map_scale.scale;
        let longitude = (ue4_coords.x - map_scale.longitude_origin) as f32 / map_scale.scale;

        Self {
            latitude,
            longitude,
            map,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::coordinates::{ArkCoordinates, ArkMap, UE4Coordinates};

    // Aberration Red Surface entrance
    const UE4_AB_RED_SURFACE: UE4Coordinates = UE4Coordinates {
        x: -172185,
        y: 229467,
        z: Some(19481),
        map: ArkMap::Aberration,
    };
    const ARK_AB_RED_SURFACE_ROUNDED: ArkCoordinates = ArkCoordinates {
        latitude: 78.7,
        longitude: 28.5,
        map: ArkMap::Aberration,
    };

    #[test]
    fn compare_distance() {

    }

    /// Test coordiante conversion from UE4 to ARK coordinates and vice versa.
    /// Will fail if:
    ///  - The map scales have changed,
    ///  - The conversion formulae are incorrect.
    #[test]
    fn convert_coords() {
        // Test conversion from UE4 to ARK Coordinates
        let ark_ab_red_surface: ArkCoordinates = UE4_AB_RED_SURFACE.into();
        assert_eq!(ark_ab_red_surface.rounded(), ARK_AB_RED_SURFACE_ROUNDED);

        // Test conversion back from ARK to UE4 coordinates then compare x, y, and the map,
        // Because z coordinates are lost in conversion from UE4 to ARK.
        let ue4_ab_red_surface: UE4Coordinates = ark_ab_red_surface.into();
        assert_eq!(ue4_ab_red_surface.x, UE4_AB_RED_SURFACE.x);
        assert_eq!(ue4_ab_red_surface.y, UE4_AB_RED_SURFACE.y);
        assert_eq!(ue4_ab_red_surface.map, UE4_AB_RED_SURFACE.map);

    }
}
