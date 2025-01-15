pub mod gui;

pub const MEAN_EARTH_RADIUS: f64 = 6371008.8;

#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point {
            x,
            y,
        }
    }

    pub fn coordinate(&self, width: f64, height: f64) -> Coordinate {
        let lon = 180.0*(2.0*self.x/(width-1.0) - 1.0);
        let lat = 90.0*(1.0 - 2.0*self.y/(height-1.0));

        Coordinate([lon, lat])
    }

    pub fn distance(&self, rhs: Point) -> f64 {
        ((self.x - rhs.x).powi(2) + (self.y - rhs.y).powi(2)).sqrt()
    }
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Coordinate([f64; 2]);

impl Coordinate {
    pub fn new(lon: f64, lat: f64) -> Self {
        Coordinate([ lon, lat ])
    }

    pub fn lon(&self) -> f64 { self.0[0] }
    pub fn lat(&self) -> f64 { self.0[1] }

    pub fn screen(&self, width: f64, height: f64) -> Point {
        let x = (width-1.0)*(self.lon()/180.0 + 1.0)/2.0;
        let y = (height-1.0)*(1.0 - self.lat()/90.0)/2.0;

        Point { x, y }
    }

    /// Tedius to implement and test, so this borrows from
    /// https://docs.rs/geo/0.11.0/geo/algorithm/haversine_distance/trait.HaversineDistance.html
    pub fn great_circle_distance(&self, rhs: Coordinate) -> f64 {
        use std::f64::consts::PI;

        let theta1 = PI*self.lat()/180.0;
        let theta2 = PI*rhs.lat()/180.0;
        let delta_theta = PI*(rhs.lat() - self.lat())/180.0;
        let delta_lambda = PI*(rhs.lon() - self.lon())/180.0;
        let a = (delta_theta / 2.0).sin().powi(2)
            + theta1.cos() * theta2.cos() * (delta_lambda / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();
        MEAN_EARTH_RADIUS * c
    }
}

