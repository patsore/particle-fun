use glam::{Vec2, Vec3};

pub struct FluidSim {
    grid_width: usize,
    grid_height: usize,

    cell_kind: Vec<u32>,
    smoke: Vec<f32>,
    velocities_x: Vec<f32>,
    velocities_y: Vec<f32>,
    pressures: Vec<f32>,
    density: f32,
    h: f32, // Cell size or spacing
}

const DT: f32 = 0.001;
const GRAVITY_VEC: Vec2 = Vec2::new(0.0, -9.8);
const OVER_RELAXATION: f32 = 1.9;

enum FieldType {
    U,
    V,
    S,
}

impl FluidSim {
    pub fn new(grid_width: usize, grid_height: usize, cell_size: f32, density: f32) -> Self {
        // Ensure that the grid dimensions are not zero
        assert!(grid_width > 0 && grid_height > 0, "Grid dimensions must be greater than zero.");

        // Initialize cell_kind with `Fluid` for all cells
        let mut cell_kind = vec![1; grid_width * grid_height];

        for x in 0..grid_width {
            // cell_kind[x] = 0; // Top edge
            cell_kind[(grid_height - 1) * grid_width + x] = 0; // Bottom edge
        }

        for y in 0..grid_height {
            cell_kind[y * grid_width] = 0; // Left edge
            cell_kind[y * grid_width + grid_width - 1] = 0; // Right edge
        }

        // Initialize smoke, velocities_x, velocities_y, and pressures with zeroes
        let smoke = vec![0.0; grid_width * grid_height];
        let velocities_x = vec![0.0; grid_width * grid_height];
        let velocities_y = vec![0.0; grid_width * grid_height];
        let pressures = vec![0.0; grid_width * grid_height];

        Self {
            grid_width,
            grid_height,
            cell_kind,
            smoke,
            velocities_x,
            velocities_y,
            pressures,
            density,
            h: cell_size, // Set the cell size (spacing) based on the provided value
        }
    }

    pub fn setup_wind_tunnel(&mut self, inflow_velocity: f32, smoke_radius: usize) {
        // Define left boundary (inlet)
        for j in 0..self.grid_height {
            let index = j * self.grid_width;
            self.cell_kind[index] = 1; // Ensure it's fluid at the boundary
            self.velocities_x[index] = inflow_velocity; // Set the inflow velocity
        }

        // Define right boundary (outlet)
        for j in 0..self.grid_height {
            let index = j * self.grid_width + (self.grid_width - 1);
            self.cell_kind[index] = 1; // Ensure it's fluid at the boundary
            self.velocities_x[index] = -inflow_velocity; // Outflow should be zero or adjusted based on your scenario
        }

        // Define top and bottom boundaries as obstacles
        for i in 0..self.grid_width {
            let bottom_index = (self.grid_height - 1) * self.grid_width + i;
            let top_index = i;
            self.cell_kind[bottom_index] = 0;
            self.cell_kind[top_index] = 0;
        }

          // Example: add some smoke in the center of the grid
        let center_x = self.grid_width / 2;
        let center_y = self.grid_height / 2;

        self.cell_kind[center_x * self.grid_width + self.grid_width - 10] = 0;
        self.cell_kind[(center_x - 1) * self.grid_width + self.grid_width - 10] = 0;
        self.cell_kind[(center_x - 2) * self.grid_width + self.grid_width - 10] = 0;
        self.cell_kind[(center_x + 1) * self.grid_width + self.grid_width - 10] = 0;
        self.cell_kind[(center_x + 2) * self.grid_width + self.grid_width - 10] = 0;

        self.add_smoke();
    }

    pub fn add_smoke(&mut self){
        let center_x = self.grid_width / 2;
        let center_y = self.grid_height / 2;

        let smoke_radius = 5;

        for i in (center_x.saturating_sub(smoke_radius))..(center_x + smoke_radius) {
                let index = i * self.grid_width + self.grid_height - 3;
                if i < self.grid_width {
                    self.smoke[index] = 1.0; // Set smoke concentration
                }
        }
    }

    pub fn simulate(&mut self, dt: f32, gravity: Vec2, num_iters: usize) {
        self.integrate(dt, gravity);

        self.pressures.fill(0.0);
        self.solve_incompressibility(num_iters);

        self.extrapolate();

        self.advect_vel(dt);

        self.advect_smoke(dt);
    }

    pub fn integrate(&mut self, dt: f32, gravity: Vec2) {
        let n = self.grid_width;

        for i in 1..self.grid_width {
            for j in 1..self.grid_height - 1 {
                let index = i * n + j;

                if self.cell_kind[index] == 1 && self.cell_kind[i * n + j - 1] == 1 {
                    self.velocities_y[index] += gravity.y * dt;
                }
            }
        }
    }

    pub fn solve_incompressibility(&mut self, num_iters: usize) {
        let n = self.grid_width;
        let cp = self.density * self.h / DT;

        for _ in 0..num_iters {
            for i in 1..self.grid_width - 1 {
                for j in 1..self.grid_height - 1 {
                    let index = i * n + j;

                    // Skip if the cell is not fluid
                    if self.cell_kind[index] != 1 {
                        continue;
                    }

                    let s = self.cell_kind[index] as f32;
                    let sx0 = self.cell_kind[(i - 1) * n + j] as f32;
                    let sx1 = self.cell_kind[(i + 1) * n + j] as f32;
                    let sy0 = self.cell_kind[i * n + j - 1] as f32;
                    let sy1 = self.cell_kind[i * n + j + 1] as f32;

                    let s_sum = sx0 + sx1 + sy0 + sy1;
                    if s_sum == 0.0 {
                        continue;
                    }

                    let div = self.velocities_x[(i + 1) * n + j] - self.velocities_x[i * n + j]
                        + self.velocities_y[i * n + j + 1] - self.velocities_y[i * n + j];

                    let p = -div / s_sum;
                    let p = p * OVER_RELAXATION;
                    self.pressures[index] += cp * p;

                    self.velocities_x[i * n + j] -= sx0 * p;
                    self.velocities_x[(i + 1) * n + j] += sx1 * p;
                    self.velocities_y[i * n + j] -= sy0 * p;
                    self.velocities_y[i * n + j + 1] += sy1 * p;
                }
            }
        }
    }

    fn sample_field(&self, x: f32, y: f32, field: FieldType) -> f32 {
        let n = self.grid_height;
        let h = self.h;
        let h1 = 1.0 / h;
        let h2 = 0.5 * h;

        let x = f32::max(f32::min(x, self.grid_width as f32 * h), h);
        let y = f32::max(f32::min(y, self.grid_width as f32 * h), h);

        let mut dx = 0.;
        let mut dy = 0.;

        let f = match field {
            FieldType::U => {
                dy = h2;
                &self.velocities_x
            }
            FieldType::V => {
                dx = h2;
                &self.velocities_y
            }
            FieldType::S => {
                dx = h2;
                dy = h2;
                &self.smoke
            }
        };

        let x0 = f32::min(f32::floor((x - dx) * h1), self.grid_width as f32 - 1.);
        let tx = ((x - dx) - x0 * h) * h1;
        let x1 = f32::min(x0 + 1., self.grid_width as f32 - 1.);

        let y0 = f32::min(f32::floor((y - dy) * h1), self.grid_height as f32 - 1.);
        let ty = ((y - dy) - y0 * h) * h1;
        let y1 = f32::min(y0 + 1., self.grid_height as f32 - 1.);

        let sx = 1. - tx;
        let sy = 1. - ty;

        sx * sy * f[x0 as usize * n + y0 as usize]
            + tx * sy * f[x1 as usize * n + y0 as usize]
            + tx * ty * f[x1 as usize * n + y1 as usize]
            + sx * ty * f[x0 as usize * n + y1 as usize]
    }

    fn avg_u(&self, i: usize, j: usize) -> f32 {
        let n = self.grid_width;
        (self.velocities_x[i * n + j] + self.velocities_x[(i + 1) * n + j]) * 0.5
    }

    fn avg_v(&self, i: usize, j: usize) -> f32 {
        let n = self.grid_width;
        (self.velocities_y[i * n + j] + self.velocities_y[i * n + j + 1]) * 0.5
    }

    pub fn advect_vel(&mut self, dt: f32) {
        let mut new_u = self.velocities_x.clone();
        let mut new_v = self.velocities_y.clone();
        let n = self.grid_width;
        let h = self.h;
        let h2 = 0.5 * h;

        for i in 1..self.grid_width {
            for j in 1..self.grid_height {
                // u component
                if self.cell_kind[i * n + j] == 1 && self.cell_kind[(i - 1) * n + j] == 1 && j < self.grid_height - 1 {
                    let mut x = i as f32 * h;
                    let mut y = j as f32 * h + h2;
                    let u = self.velocities_x[i * n + j];
                    let v = self.avg_v(i, j);
                    x -= dt * u;
                    y -= dt * v;
                    let sampled_u = self.sample_field(x, y, FieldType::U);
                    new_u[i * n + j] = sampled_u;
                }
                // v component
                if self.cell_kind[i * n + j] == 1 && self.cell_kind[i * n + j - 1] == 1 && i < self.grid_width - 1 {
                    let mut x = i as f32 * h + h2;
                    let mut y = j as f32 * h;
                    let u = self.avg_u(i, j);
                    let v = self.velocities_y[i * n + j];
                    x -= dt * u;
                    y -= dt * v;
                    let sampled_v = self.sample_field(x, y, FieldType::V);
                    new_v[i * n + j] = sampled_v;
                }
            }
        }

        self.velocities_x = new_u;
        self.velocities_y = new_v;
    }

    pub fn advect_smoke(&mut self, dt: f32) {
        let mut new_m = self.smoke.clone(); // Assuming `smoke` is a Vec<f32> field
        let n = self.grid_width;
        let h = self.h;
        let h2 = 0.5 * h;

        for i in 1..self.grid_width - 1 {
            for j in 1..self.grid_height - 1 {
                if self.cell_kind[i * n + j] == 1 {
                    let u = (self.velocities_x[i * n + j] + self.velocities_x[(i + 1) * n + j]) * 0.5;
                    let v = (self.velocities_y[i * n + j] + self.velocities_y[i * n + j + 1]) * 0.5;
                    let mut x = i as f32 * h + h2 ;
                    let mut y = j as f32 * h + h2;
                    // println!("{x}, {y}");
                    x -= dt * u;
                    y -= dt * v;
                    // println!("{x}, {y}");
                    let test = self.sample_field(x, y, FieldType::S);
                    if test != 0.0{
                        // println!("{test}, {x}, {y}");
                        new_m[i * n + j] = test;
                    }
                }
            }
        }
        self.smoke = new_m;
    }

    pub fn to_vectors(&self) -> Vec<crate::vector::Vector> {
        let mut vectors = Vec::with_capacity(self.grid_width * self.grid_height);

        for i in 0..self.grid_width {
            for j in 0..self.grid_height {
                let index = i * self.grid_width + j;
                let start = Vec3::new(
                    i as f32 * self.h,
                    0.0,
                    j as f32 * self.h,
                );

                if self.cell_kind[index] != 1 {
                    let direction = Vec3::NEG_Y;
                    vectors.push(crate::vector::Vector::new(start, direction, 5.0));
                    continue;
                }

                let direction = Vec3::new(
                    self.velocities_x[index],
                    0.0,
                    self.velocities_y[index],
                );
                let magnitude = self.smoke[index];

                vectors.push(crate::vector::Vector::new(start, direction.normalize(), magnitude));
            }
        }

        vectors
    }

    pub fn extrapolate(&mut self) {
        let n = self.grid_width;
        let num_y = self.grid_height;

        // Extrapolate u velocity on the boundaries
        for i in 0..self.grid_width {
            self.velocities_x[i * num_y + 0] = self.velocities_x[i * num_y + 1];
            self.velocities_x[i * num_y + num_y - 1] = self.velocities_x[i * num_y + num_y - 2];
        }

        // Extrapolate v velocity on the boundaries
        for j in 0..num_y {
            self.velocities_y[0 * num_y + j] = self.velocities_y[1 * num_y + j];
            self.velocities_y[(self.grid_width - 1) * num_y + j] = self.velocities_y[(self.grid_width - 2) * num_y + j];
        }
    }
}