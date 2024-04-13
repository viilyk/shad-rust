use crate::data::{Cell, Direction, World};
use std::cmp::min;

////////////////////////////////////////////////////////////////////////////////

pub struct Strategy {
    previous_direction: Direction,
    angle_cell: Cell,
    start_cell: Cell,
    clock: bool,
}

impl Default for Strategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy {
    pub fn new() -> Self {
        Self {
            previous_direction: Direction::Up,
            angle_cell: Cell(0, 0),
            start_cell: Cell(0, 0),
            clock: true,
        }
    }

    pub fn on_tick(&mut self, world: World) -> Direction {
        let player = world.me();
        let position_cell = player.position.to_cell();
        //eprintln!("AAAATerritory: {:?}", player.territory);
        eprintln!("Position: {:?}", position_cell);
        //eprintln!("Angle: {:?}", self.angle_cell);
        //eprintln!("Previous direction {:?}", self.previous_direction);
        if !player.territory.contains(&player.position) {
            eprintln!("Angle: {:?}", self.angle_cell);
            if position_cell == self.angle_cell {
                self.angle_cell = self.start_cell;
                self.previous_direction = self.previous_direction.next(self.clock);
                //eprintln!("{}", self.previous_direction as i32);
                return self.previous_direction;
            }
            if (self.previous_direction as i32) % 2 == 1 && self.angle_cell.0 == position_cell.0
                || (self.previous_direction as i32) % 2 == 0 && self.angle_cell.1 == position_cell.1
            {
                self.previous_direction = self.previous_direction.next(self.clock);
                //eprintln!("{}", self.previous_direction as i32);
                return self.previous_direction;
            } else {
                //eprintln!("{}", self.previous_direction as i32);
                return self.previous_direction;
            }
        }

        let iter_territory = player.territory.clone().into_iter().map(|v| v.to_cell());

        let mut rectangle_scores: Vec<(Cell, i32)> = world
            .iter_cells()
            .map(|c| {
                // let perimeter = 2 * position_cell.distance_to(c) + 4;
                let perimeter = 2 * position_cell.distance_to(c);
                let mut k = 0;
                let g = iter_territory
                    .clone()
                    .filter(|p| p.between(c, position_cell))
                    .count();
                // eprintln!("g: {}", g);
                let distances_to_enems: Vec<i32> = world
                    .iter_enemies()
                    .map(|enem| {
                        let position_enem = enem.1.position.to_cell();
                        k += enem
                            .1
                            .territory
                            .clone()
                            .into_iter()
                            .map(|v| v.to_cell())
                            .filter(|p| p.between(position_cell, c))
                            .count();

                        let x_distance =
                            ((position_cell.0 - position_enem.0), (c.0 - position_enem.0));
                        let y_distance =
                            ((position_cell.1 - position_enem.1), (c.1 - position_enem.1));
                        if x_distance.0 * x_distance.1 < 0 {
                            if y_distance.0 * y_distance.1 < 0 {
                                return min(
                                    min(x_distance.0.abs(), x_distance.1.abs()),
                                    min(y_distance.0.abs(), y_distance.1.abs()),
                                );
                            }
                            return min(y_distance.0.abs(), y_distance.1.abs());
                        }
                        if y_distance.0 * y_distance.1 < 0 {
                            return min(x_distance.0.abs(), x_distance.1.abs());
                        }

                        min(x_distance.0.abs(), x_distance.1.abs())
                            + min(y_distance.0.abs(), y_distance.1.abs())
                    })
                    .collect();
                // let bonus = world
                //     .iter_enemies()
                //     .map()
                //eprintln!("Distances: {:?} -- {:?}", c, distances_to_enems);
                let min_distance = *distances_to_enems.iter().min().unwrap();
                let danger = perimeter - min_distance;
                let cells_score = ((c.0 - position_cell.0).abs()) * ((c.1 - position_cell.1).abs())
                    - (g as i32)
                    + (k as i32) * 4;
                (c, cells_score - danger * danger)
            })
            .collect();
        //eprintln!("Rectangles: {:?}", rectangle_scores);
        let mut max_rectangle = rectangle_scores[0].1;
        let mut max_cell = rectangle_scores[0].0;
        while let Some((x, y)) = rectangle_scores.pop() {
            if y > max_rectangle {
                max_rectangle = y;
                max_cell = x;
            }
        }
        self.angle_cell = max_cell;
        self.start_cell = position_cell;
        eprintln!("{:?}", max_cell);
        let direction = position_cell.directions_to(max_cell);
        //eprintln!("{:?}", direction);
        if direction.0 == self.previous_direction.opposite() {
            self.clock = !direction.1;
            self.previous_direction = direction.0.next(direction.1);
            //eprintln!("{}", self.previous_direction as i32);
            self.previous_direction
        } else {
            self.clock = direction.1;
            self.previous_direction = direction.0;
            //eprintln!("{}", self.previous_direction as i32);
            self.previous_direction
        }
    }
}
