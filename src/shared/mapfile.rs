use byteorder::{LittleEndian, ReadBytesExt};
use na::Vector3;
use na::Vector2;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::error::Error;
use shared::calc;

const MAX_POLYS: i32 = 5000;
//const MIN_SECTOR: i32 = -25;
const MAX_SECTOR: i32 = 25;
//const MIN_SECTORZ: i32 = -35;
//const MAX_SECTORZ: i32 = 35;
//const TILESECTOR: i32 = 3;
//const MIN_TILE: i32 = MIN_SECTOR * TILESECTOR;
//const MAX_TILE: i32 = MAX_SECTOR * TILESECTOR;
const MAX_PROPS: i32 = 500;
//const MAX_SPAWNPOINTS: i32 = 255;
//const MAX_COLLIDERS: i32 = 128;

#[derive(Debug, Copy, Clone)]
pub struct MapColor {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub a: u8,
}
#[derive(Debug, Copy, Clone)]
pub struct MapVertex {
  pub x: f32,
  pub y: f32,
  pub z: f32,
  pub rhw: f32,
  pub color: MapColor,
  pub u: f32,
  pub v: f32,
}
#[derive(Debug, Copy, Clone)]
pub struct MapPolygon {
  pub vertices: [MapVertex; 3],
  normals: [Vector3<f32>; 3],
  polytype: u8,
}
#[derive(Debug, Clone, Default)]
pub struct MapSector {
  pub polys: Vec<u16>,
}

#[derive(Debug)]
pub struct MapProp {
  pub active: bool,
  pub style: u16,
  pub width: i32,
  pub height: i32,
  pub x: f32,
  pub y: f32,
  pub rotation: f32,
  pub scale_x: f32,
  pub scale_y: f32,
  pub alpha: u8,
  pub color: MapColor,
  pub level: u8,
}
#[derive(Debug)]
pub struct MapScenery {
  pub filename: String,
  date: i32,
}
#[derive(Debug)]
pub struct MapCollider {
  active: bool,
  x: f32,
  y: f32,
  radius: f32,
}
#[derive(Debug)]
pub struct MapSpawnpoint {
  pub active: bool,
  pub x: i32,
  pub y: i32,
  pub team: i32,
}
#[allow(dead_code)]
pub struct MapFile {
  filename: String,
  version: i32,
  mapname: String,
  pub texture_name: String,
  pub bg_color_top: MapColor,
  pub bg_color_bottom: MapColor,
  pub start_jet: i32,
  grenade_packs: u8,
  medikits: u8,
  weather: u8,
  steps: u8,
  random_id: i32,
  pub polygons: Vec<MapPolygon>,
  pub sectors_division: i32,
  pub sectors_num: i32,
  sectors: Vec<MapSector>,
  pub props: Vec<MapProp>,
  pub scenery: Vec<MapScenery>,
  colliders: Vec<MapCollider>,
  pub spawnpoints: Vec<MapSpawnpoint>,
  pub sectors_poly: Vec<Vec<MapSector>>,
  pub perps: Vec<[Vector2<f32>; 3]>,
}
impl MapFile {
  pub fn load_map_file(file_name: &str) -> MapFile {
    let mut path = PathBuf::new();
    path.push("assets/maps/");
    path.push(file_name);
    let file = File::open(&path).expect("Error opening File");
    let mut buf = BufReader::new(file);

    let filename = path.to_string_lossy().into_owned();
    let version = buf.read_i32::<LittleEndian>().unwrap();

    let mapname = read_string(&mut buf, 38).ok().unwrap();
    let texture_name = read_string(&mut buf, 24).ok().unwrap();
    let bg_color_top = read_color(&mut buf);

    let bg_color_bottom = read_color(&mut buf);

    let start_jet = buf.read_i32::<LittleEndian>().unwrap();

    let grenade_packs = buf.read_u8().unwrap();

    let medikits = buf.read_u8().unwrap();

    let weather = buf.read_u8().unwrap();

    let steps = buf.read_u8().unwrap();

    let random_id = buf.read_i32::<LittleEndian>().unwrap();
    let n = buf.read_i32::<LittleEndian>().unwrap();
    if (n > MAX_POLYS) || (n < 0) {
      panic!("Wrong PMS data (number of polygons)");
    }

    let mut polygons: Vec<MapPolygon> = Vec::new();
    let mut perps = Vec::new();
    for _i in 0..n {
      let mut vertices: [MapVertex; 3] = [
        read_vertex(&mut buf),
        read_vertex(&mut buf),
        read_vertex(&mut buf),
      ];
      let normals: [Vector3<f32>; 3] = [
        read_vec3(&mut buf),
        read_vec3(&mut buf),
        read_vec3(&mut buf),
      ];
      let polytype = buf.read_u8().unwrap();

      polygons.push(MapPolygon {
        vertices: vertices,
        normals: normals,
        polytype: polytype,
      });

      let mut perp: [Vector2<f32>; 3] = [
        Vector2::new(normals[0].x, normals[0].y),
        Vector2::new(normals[1].x, normals[1].y),
        Vector2::new(normals[2].x, normals[2].y),
      ];
      perp[0] = calc::vec2normalize(perp[0], perp[0]);
      perp[1] = calc::vec2normalize(perp[1], perp[1]);
      perp[2] = calc::vec2normalize(perp[2], perp[2]);
      perps.push(perp);
    }
    let sectors_division = buf.read_i32::<LittleEndian>().unwrap();

    let sectors_num = buf.read_i32::<LittleEndian>().unwrap();

    if (sectors_num > MAX_SECTOR) || (sectors_num < 0) {
      panic!("Wrong PMS data (number of sectors)");
    }

    let n = (2 * sectors_num + 1) * (2 * sectors_num + 1);

    let mut sectors: Vec<MapSector> = Vec::new();

    for _i in 0..n {
      let m = buf.read_u16::<LittleEndian>().unwrap();

      if m as i32 > MAX_POLYS {
        break;
      }
      let mut polys: Vec<u16> = Vec::new();

      for _j in 0..m {
        polys.push(buf.read_u16::<LittleEndian>().unwrap());
      }
      sectors.push(MapSector { polys });
    }

    let mut k = 0;
    let sector = MapSector { polys: Vec::new() };

    let sectores = vec![sector.clone(); 51];
    let mut sectored = vec![sectores.clone(); 71];
    for i in 0..51 {
      for j in 0..51 {
        sectored[i][j] = sectors[k].clone();
        k += 1;
      }
    }
    let sectors_poly = sectored;

    let n = buf.read_i32::<LittleEndian>().unwrap();
    if (n > MAX_PROPS) || (n < 0) {
      panic!("Wrong PMS data (number of props)");
    }

    let mut props: Vec<MapProp> = Vec::new();

    for _i in 0..n {
      let active = buf.read_u16::<LittleEndian>().unwrap() != 0;
      let style = buf.read_u16::<LittleEndian>().unwrap();
      let width = buf.read_i32::<LittleEndian>().unwrap();
      let height = buf.read_i32::<LittleEndian>().unwrap();
      let x = buf.read_f32::<LittleEndian>().unwrap();
      let y = buf.read_f32::<LittleEndian>().unwrap();
      let rotation = buf.read_f32::<LittleEndian>().unwrap();
      let scale_x = buf.read_f32::<LittleEndian>().unwrap();
      let scale_y = buf.read_f32::<LittleEndian>().unwrap();
      let alpha = buf.read_i32::<LittleEndian>().unwrap() as u8;
      let mut color = read_color(&mut buf);
      color.a = alpha;
      let level = buf.read_i32::<LittleEndian>().unwrap() as u8;
      props.push(MapProp {
        active,
        style,
        width,
        height,
        x,
        y,
        rotation,
        scale_x,
        scale_y,
        alpha,
        color,
        level,
      });
    }
    let n = buf.read_i32::<LittleEndian>().unwrap();

    let mut scenery: Vec<MapScenery> = Vec::new();

    for _i in 0..n {
      let mut filename = read_string(&mut buf, 50).ok().unwrap();
      let date = buf.read_i32::<LittleEndian>().unwrap();
      scenery.push(MapScenery {
        filename: filename,
        date,
      });
    }
    let n = buf.read_i32::<LittleEndian>().unwrap();

    let mut colliders: Vec<MapCollider> = Vec::new();

    for _i in 0..n {
      let active = buf.read_i32::<LittleEndian>().unwrap() != 0;
      let x = buf.read_f32::<LittleEndian>().unwrap();
      let y = buf.read_f32::<LittleEndian>().unwrap();
      let radius = buf.read_f32::<LittleEndian>().unwrap();
      colliders.push(MapCollider {
        active,
        x,
        y,
        radius,
      });
    }

    let n = buf.read_i32::<LittleEndian>().unwrap();

    let mut spawnpoints: Vec<MapSpawnpoint> = Vec::new();

    for _i in 0..n {
      let active = buf.read_i32::<LittleEndian>().unwrap() != 0;
      let x = buf.read_i32::<LittleEndian>().unwrap();
      let y = buf.read_i32::<LittleEndian>().unwrap();
      let team = buf.read_i32::<LittleEndian>().unwrap();
      spawnpoints.push(MapSpawnpoint { active, x, y, team });
    }

    MapFile {
      filename,
      version,
      mapname,
      texture_name,
      bg_color_top,
      bg_color_bottom,
      start_jet,
      grenade_packs,
      medikits,
      weather,
      steps,
      random_id,
      polygons,
      sectors_division,
      sectors_num,
      sectors,
      props,
      scenery,
      colliders,
      spawnpoints,
      sectors_poly,
      perps,
    }
  }
  pub fn point_in_poly(&mut self, p: Vector2<f32>, poly: &mut MapPolygon) -> bool {
    let a = &poly.vertices[0];
    let b = &poly.vertices[1];
    let c = &poly.vertices[2];

    let ap_x = p.x - a.x;
    let ap_y = p.y - a.y;
    let p_ab = (b.x - a.x) * ap_y - (b.y - a.y) * ap_x > 0.0f32;
    let p_ac = (c.x - a.x) * ap_y - (c.y - a.y) * ap_x > 0.0f32;

    if p_ac == p_ab {
      return false;
    }

    if ((c.x - b.x) * (p.y - b.y) - (c.y - b.y) * (p.x - b.x) > 0.0f32) != p_ab {
      return false;
    }

    true
  }
  pub fn closest_perpendicular(
    &mut self,
    j: i32,
    pos: Vector2<f32>,
    d: &mut f32,
    n: &mut i32,
  ) -> Vector2<f32> {
    let px: [f32; 3] = [
      self.polygons[j as usize].vertices[0].x,
      self.polygons[j as usize].vertices[1].x,
      self.polygons[j as usize].vertices[2].x,
    ];

    let py: [f32; 3] = [
      self.polygons[j as usize].vertices[0].y,
      self.polygons[j as usize].vertices[1].y,
      self.polygons[j as usize].vertices[2].y,
    ];

    let mut p1 = Vector2::new(px[0], py[0]);

    let mut p2 = Vector2::new(px[1], py[1]);

    let d1 = calc::point_line_distance(p1, p2, pos);

    *d = d1;

    let mut edge_v1 = 1;
    let mut edge_v2 = 2;

    p1.x = px[1];
    p1.y = py[1];

    p2.x = px[2];
    p2.y = py[2];

    let d2 = calc::point_line_distance(p1, p2, pos);

    if d2 < d1 {
      edge_v1 = 2;
      edge_v2 = 3;
      *d = d2;
    }

    p1.x = px[2];
    p1.y = py[2];

    p2.x = px[0];
    p2.y = py[0];

    let d3 = calc::point_line_distance(p1, p2, pos);

    if (d3 < d2) && (d3 < d1) {
      edge_v1 = 3;
      edge_v2 = 1;
      *d = d3;
    }

    if edge_v1 == 1 && edge_v2 == 2 {
      *n = 1;
      return self.perps[j as usize][0];
    }
    if edge_v1 == 2 && edge_v2 == 3 {
      *n = 2;
      return self.perps[j as usize][1];
    }
    if edge_v1 == 3 && edge_v2 == 1 {
      *n = 3;
      return self.perps[j as usize][2];
    }

    Vector2::new(0.0f32, 0.0f32)
  }
}
pub fn read_string<T: Read>(reader: &mut T, length: u32) -> Result<String, Box<Error>> {
  let mut buffer: Vec<u8>;
  let byte = reader.read_u8()?;
  buffer = vec![0u8; byte as usize];
  reader.read_exact(buffer.as_mut_slice())?;

  let filler = length - u32::from(byte);
  for _i in 0..filler {
    let _ = reader.read_u8()?;
  }

  let x = String::from_utf8_lossy(&buffer).into_owned();

  Ok(x)
}
pub fn read_color<T: Read>(reader: &mut T) -> MapColor {
  let b = reader.read_u8().unwrap();
  let g = reader.read_u8().unwrap();
  let r = reader.read_u8().unwrap();
  let a = reader.read_u8().unwrap();

  MapColor { r, g, b, a }
}
pub fn read_vertex<T: Read>(reader: &mut T) -> MapVertex {
  let x = reader.read_f32::<LittleEndian>().unwrap();
  let y = reader.read_f32::<LittleEndian>().unwrap();
  let z = reader.read_f32::<LittleEndian>().unwrap();
  let rhw = reader.read_f32::<LittleEndian>().unwrap();
  let color = read_color(reader);
  let u = reader.read_f32::<LittleEndian>().unwrap();
  let v = reader.read_f32::<LittleEndian>().unwrap();

  MapVertex {
    x,
    y,
    z,
    rhw,
    color,
    u,
    v,
  }
}
pub fn read_vec3<T: Read>(reader: &mut T) -> Vector3<f32> {
  let x = reader.read_f32::<LittleEndian>().unwrap();
  let y = reader.read_f32::<LittleEndian>().unwrap();
  let z = reader.read_f32::<LittleEndian>().unwrap();

  Vector3::new(x, y, z)
}
