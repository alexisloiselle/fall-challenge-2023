use std::{
    cmp,
    collections::{HashMap, HashSet},
    io,
};

use rand::seq::SliceRandom;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

// Ideas

// Scoring heuristic based on the game description

// Minimax as it's a zero sum game
// Alpha beta pruning to reduce the number of nodes to explore
// To use when the strategy should be deterministic

// MCTS could be useful because of the number of possible states
// Relevant because of the randomness in which fishes move
// To use when the strategy should be stochastic

const MOVE_SPEED: f64 = 600.0;
const SINK_SPEED: f64 = 300.0;
const LIGHT_BASE_RADIUS: f64 = 800.0;
const LIGHT_POWER_RADIUS: f64 = 2000.0;
const MIN_BATTERY: i32 = 0;
const MAX_BATTERY: i32 = 30;

#[derive(Clone, Debug)]
struct Move {
    should_move: bool,
    x: Option<i32>,
    y: Option<i32>,
    light: bool,
}
impl Move {
    fn clone(&self) -> Move {
        Move {
            should_move: self.should_move,
            x: self.x,
            y: self.y,
            light: self.light,
        }
    }
}

#[derive(Clone, Debug)]
struct Creature {
    id: i32,
    color: i32,
    x: Option<i32>,
    y: Option<i32>,
    vx: Option<i32>,
    vy: Option<i32>,
    _type: i32,
}
impl Creature {
    fn clone(&self) -> Creature {
        Creature {
            id: self.id,
            color: self.color,
            x: self.x,
            y: self.y,
            vx: self.vx,
            vy: self.vy,
            _type: self._type,
        }
    }

    pub fn get_score(&self) -> i32 {
        if self._type == 0 {
            1
        } else if self._type == 1 {
            2
        } else if self._type == 2 {
            3
        } else {
            0
        }
    }
}

#[derive(Clone, Debug)]
struct Drone {
    id: i32,
    x: i32,
    y: i32,
    emergency: i32,
    battery: i32,
    is_mine: bool,
}
impl Drone {
    fn clone(&self) -> Drone {
        Drone {
            id: self.id,
            x: self.x,
            y: self.y,
            emergency: self.emergency,
            battery: self.battery,
            is_mine: self.is_mine,
        }
    }

    fn distance_from(&self, x: f64, y: f64) -> f64 {
        ((self.x as f64 - x).powf(2.0) + (self.y as f64 - y).powf(2.0)).sqrt()
    }

    fn is_near_creature(&self, creature: &Creature) -> bool {
        let distance = self.distance_from(creature.x.unwrap() as f64, creature.y.unwrap() as f64);
        distance <= LIGHT_BASE_RADIUS
    }

    fn is_near_creature_with_power(&self, creature: &Creature) -> bool {
        let distance = self.distance_from(creature.x.unwrap() as f64, creature.y.unwrap() as f64);
        distance <= LIGHT_POWER_RADIUS
    }
}

#[derive(Clone, Debug)]
struct RadarBlip {
    drone_id: i32,
    creature_id: i32,
    radar: String,
}
impl RadarBlip {
    fn clone(&self) -> RadarBlip {
        RadarBlip {
            drone_id: self.drone_id,
            creature_id: self.creature_id,
            radar: self.radar.clone(),
        }
    }
}

fn normalize_vector(x: f64, y: f64) -> (f64, f64) {
    let norm = (x.powf(2.0) + y.powf(2.0)) as f64;
    let norm = norm.sqrt();
    let x = x / norm;
    let y = y / norm;
    (x, y)
}

fn emphasize_value(x: f64) -> f64 {
    // Constants (these may need tuning)
    let a = 1500.0;
    let b = 1.05;
    let c = 1.0;
    let d = -1500.0;

    a * (x + c).log(b) + d
}

#[derive(Clone, Debug)]
struct GameState {
    was_type_0_achieved: bool,
    was_type_1_achieved: bool,
    was_type_2_achieved: bool,
    was_1_of_each_achieved: bool,
    was_all_colors_achieved: bool,
    my_score: i32,
    foe_score: i32,
    my_scan_count: i32,
    foe_scan_count: i32,
    my_drone_count: i32,
    foe_drone_count: i32,
    creatures: HashMap<i32, Creature>,
    my_drones: HashMap<i32, Drone>,
    their_drones: HashMap<i32, Drone>,
    radar_blips: HashMap<i32, RadarBlip>,
    scans: HashSet<String>,
}
impl GameState {
    fn clone(&self) -> GameState {
        GameState {
            was_type_0_achieved: self.was_type_0_achieved,
            was_type_1_achieved: self.was_type_1_achieved,
            was_type_2_achieved: self.was_type_2_achieved,
            was_1_of_each_achieved: self.was_1_of_each_achieved,
            was_all_colors_achieved: self.was_all_colors_achieved,
            my_score: self.my_score,
            foe_score: self.foe_score,
            my_scan_count: self.my_scan_count,
            foe_scan_count: self.foe_scan_count,
            my_drone_count: self.my_drone_count,
            foe_drone_count: self.foe_drone_count,
            creatures: self.creatures.clone(),
            my_drones: self.my_drones.clone(),
            their_drones: self.their_drones.clone(),
            radar_blips: self.radar_blips.clone(),
            scans: self.scans.clone(),
        }
    }

    fn new() -> GameState {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let creature_count = parse_input!(input_line, i32);

        let mut creatures = HashMap::new();
        let my_drones = HashMap::new();
        let their_drones = HashMap::new();
        let radar_blips = HashMap::new();
        let scans = HashSet::new();

        for _i in 0..creature_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let creature_id = parse_input!(inputs[0], i32);
            let color = parse_input!(inputs[1], i32);
            let _type = parse_input!(inputs[2], i32);
            creatures.insert(
                creature_id,
                Creature {
                    id: creature_id,
                    color,
                    x: None,
                    y: None,
                    vx: None,
                    vy: None,
                    _type,
                },
            );
        }

        GameState {
            creatures,
            my_score: 0,
            foe_score: 0,
            my_scan_count: 0,
            foe_scan_count: 0,
            my_drone_count: 1,  // In wood league, we only have one drone
            foe_drone_count: 1, // In wood league, we only have one drone
            my_drones,
            their_drones,
            radar_blips,
            scans,
            was_type_0_achieved: false,
            was_type_1_achieved: false,
            was_type_2_achieved: false,
            was_1_of_each_achieved: false,
            was_all_colors_achieved: false,
        }
    }

    fn update_state(&mut self) {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        self.my_score = parse_input!(input_line, i32);

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        self.foe_score = parse_input!(input_line, i32);

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        self.my_scan_count = parse_input!(input_line, i32);

        for _i in 0..self.my_scan_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();

            let creature_id = parse_input!(input_line, i32);
            let my_drone_id = self.my_drones.values().next().unwrap().id;
            self.scans
                .insert(format!("{}:{}", my_drone_id, creature_id));
        }

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        self.foe_scan_count = parse_input!(input_line, i32);

        for _i in 0..self.foe_scan_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();

            let creature_id = parse_input!(input_line, i32);
            let foe_drone_id = self.their_drones.values().next().unwrap().id;
            self.scans
                .insert(format!("{}:{}", foe_drone_id, creature_id));
        }

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        self.my_drone_count = parse_input!(input_line, i32);

        for _i in 0..self.my_drone_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let drone_id = parse_input!(inputs[0], i32);

            let drone = Drone {
                id: drone_id,
                x: parse_input!(inputs[1], i32),
                y: parse_input!(inputs[2], i32),
                emergency: parse_input!(inputs[3], i32),
                battery: parse_input!(inputs[4], i32),
                is_mine: true,
            };
            self.my_drones.insert(drone_id, drone.clone());
        }

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        self.foe_drone_count = parse_input!(input_line, i32);

        for _i in 0..self.foe_drone_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let drone_id = parse_input!(inputs[0], i32);

            let drone = Drone {
                id: drone_id,
                x: parse_input!(inputs[1], i32),
                y: parse_input!(inputs[2], i32),
                emergency: parse_input!(inputs[3], i32),
                battery: parse_input!(inputs[4], i32),
                is_mine: false,
            };
            self.their_drones.insert(drone_id, drone.clone());
        }

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let drone_scan_count = parse_input!(input_line, i32);

        // This is useless
        for _i in 0..drone_scan_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();

            let _drone_id = parse_input!(inputs[0], i32);
            let _creature_id = parse_input!(inputs[1], i32);
            // self.scans.insert(format!("{}:{}", drone_id, creature_id));
        }

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let visible_creature_count = parse_input!(input_line, i32);

        for _i in 0..visible_creature_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let creature_id = parse_input!(inputs[0], i32);

            self.creatures.insert(
                creature_id,
                Creature {
                    id: creature_id,
                    x: Some(parse_input!(inputs[1], i32)),
                    y: Some(parse_input!(inputs[2], i32)),
                    vx: Some(parse_input!(inputs[3], i32)),
                    vy: Some(parse_input!(inputs[4], i32)),
                    ..self.creatures.get(&creature_id).unwrap().clone()
                },
            );
        }

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let radar_blip_count = parse_input!(input_line, i32);

        for _i in 0..radar_blip_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let drone_id = parse_input!(inputs[0], i32);

            self.radar_blips.insert(
                drone_id,
                RadarBlip {
                    drone_id,
                    creature_id: parse_input!(inputs[1], i32),
                    radar: inputs[2].trim().to_string(),
                },
            );
        }
    }

    fn minimax(&self, depth: i32, alpha: f64, beta: f64, maximizing_player: bool) -> f64 {
        if depth == 0 {
            let score = self.evaluate(None);
            return score;
        }

        if maximizing_player {
            let mut alpha = alpha;
            for moves in self.get_possible_moves() {
                let mut new_state = self.clone(); // Implement Clone for GameState or find another way to get new state
                new_state.apply_moves(moves);
                let score = new_state.minimax(depth - 1, alpha, beta, false);
                alpha = f64::max(alpha, score);
                if beta <= alpha {
                    break;
                }
            }
            alpha
        } else {
            let mut beta = beta;
            for moves in self.get_possible_moves() {
                let mut new_state = self.clone(); // Implement Clone for GameState or find another way to get new state
                new_state.apply_moves(moves);
                let score = new_state.minimax(depth - 1, alpha, beta, true);
                beta = f64::min(beta, score);
                if beta <= alpha {
                    break;
                }
            }
            beta
        }
    }

    fn evaluate(&self, log_avg: Option<bool>) -> f64 {
        let mut score = 0.0;

        score += self.my_score as f64 * 100000.0;
        score -= self.foe_score as f64 * 100000.0;

        score += if self.was_all_colors_achieved {
            500.0
        } else {
            0.0
        };

        score += if self.was_1_of_each_achieved {
            500.0
        } else {
            0.0
        };

        let my_drone = self.my_drones.values().next().unwrap();
        let avg_distance_from_creatures_not_scanned =
            self.creatures.values().fold(0.0, |acc, creature| {
                let was_scanned = self
                    .scans
                    .contains(&format!("{}:{}", my_drone.id, creature.id));

                if !was_scanned {
                    let distance_from_creature = my_drone
                        .distance_from(creature.x.unwrap() as f64, creature.y.unwrap() as f64);
                    if log_avg.unwrap_or(false) {
                        // eprintln!("distance_from_creature: {}", distance_from_creature);
                    }
                    acc + distance_from_creature
                } else {
                    acc
                }
            }) / self.creatures.len() as f64;

        let emphasized_avg_distance_from_creatures_not_scanned =
            emphasize_value(avg_distance_from_creatures_not_scanned);

        if log_avg.unwrap_or(false) {
            // eprintln!(
            //     "avg_distance_from_creatures_not_scanned: {}",
            //     avg_distance_from_creatures_not_scanned
            // );
            // eprintln!(
            //     "emphasized_avg_distance_from_creatures_not_scanned: {}",
            //     emphasized_avg_distance_from_creatures_not_scanned
            // );
        }

        score -= emphasized_avg_distance_from_creatures_not_scanned;

        let foe_drone = self.their_drones.values().next().unwrap();
        let foe_avg_distance_from_creatures_not_scanned =
            self.creatures.values().fold(0.0, |acc, creature| {
                let was_scanned = self
                    .scans
                    .contains(&format!("{}:{}", foe_drone.id, creature.id));

                if !was_scanned {
                    acc + foe_drone
                        .distance_from(creature.x.unwrap() as f64, creature.y.unwrap() as f64)
                } else {
                    acc
                }
            }) / self.creatures.len() as f64;

        let foe_emphasized_avg_distance_from_creatures_not_scanned =
            emphasize_value(foe_avg_distance_from_creatures_not_scanned);

        score += foe_emphasized_avg_distance_from_creatures_not_scanned;

        score
    }

    fn get_possible_moves(&self) -> Vec<Move> {
        let mut possible_moves = Vec::new();

        let my_drone = self.my_drones.values().next().unwrap();

        let directions = vec![
            (
                my_drone.x + MOVE_SPEED as i32,
                my_drone.y + MOVE_SPEED as i32,
            ),
            (
                my_drone.x - MOVE_SPEED as i32,
                my_drone.y - MOVE_SPEED as i32,
            ),
            (
                my_drone.x + MOVE_SPEED as i32,
                my_drone.y - MOVE_SPEED as i32,
            ),
            (
                my_drone.x - MOVE_SPEED as i32,
                my_drone.y + MOVE_SPEED as i32,
            ),
        ];
        let light_values = vec![true, false];
        for direction in directions {
            for light in light_values.clone() {
                let m = Move {
                    should_move: true,
                    x: Some(i32::max(0, i32::min(10000, direction.0))),
                    y: Some(i32::max(0, i32::min(10000, direction.1))),
                    light,
                };

                possible_moves.push(m);
            }
        }

        for light in light_values {
            let m = Move {
                should_move: false,
                x: None,
                y: None,
                light,
            };

            possible_moves.push(m);
        }

        possible_moves
    }

    fn apply_moves(&mut self, m: Move) {
        for creature in self.creatures.values_mut() {
            creature.x = creature.x.map(|x| x + creature.vx.unwrap());
            creature.y = creature.y.map(|y| y + creature.vy.unwrap());
        }

        let mut scan_info = Vec::new();
        let drone_id = self.my_drones.values().next().unwrap().id;

        let drone = self.my_drones.get_mut(&drone_id).unwrap();
        if m.should_move {
            let (normalized_x, normalized_y) = normalize_vector(
                m.x.unwrap() as f64 - drone.x as f64,
                m.y.unwrap() as f64 - drone.y as f64,
            );
            drone.x += (normalized_x * MOVE_SPEED) as i32;
            drone.y += (normalized_y * MOVE_SPEED) as i32;
        } else {
            drone.y += SINK_SPEED as i32;
        }

        drone.battery = if m.light {
            cmp::max(MIN_BATTERY, drone.battery - 5)
        } else {
            cmp::min(MAX_BATTERY, drone.battery + 1)
        };

        let mut scanned_creature_ids = Vec::new();
        for creature in self.creatures.values() {
            let was_scanned_already = self
                .scans
                .contains(&format!("{}:{}", drone_id, creature.id));

            if (drone.is_near_creature(creature)
                || (drone.is_near_creature_with_power(creature) && m.light))
                && !was_scanned_already
            {
                scanned_creature_ids.push(creature.id);
            }
        }

        scan_info.push(scanned_creature_ids);

        for scanned_creature_ids in scan_info {
            let drone = self.my_drones.get(&drone_id).unwrap();

            for creature_id in scanned_creature_ids {
                let creature = self.creatures.get(&creature_id).unwrap();

                self.scans.insert(format!("{}:{}", drone_id, creature_id));

                let mut score = 0;
                let mut creature_score = creature.get_score();
                if creature._type == 0 && !self.was_type_0_achieved {
                    self.was_type_0_achieved = true;
                    creature_score *= 2;
                } else if creature._type == 1 && !self.was_type_1_achieved {
                    self.was_type_1_achieved = true;
                    creature_score *= 2;
                } else if creature._type == 2 && !self.was_type_2_achieved {
                    self.was_type_2_achieved = true;
                    creature_score *= 2;
                }

                score += creature_score;

                let mut all_colors_score = 0;

                if self.has_scanned_all_creatures_of_color_for(creature.color, drone_id) {
                    all_colors_score += 3;
                }
                if !self.was_all_colors_achieved {
                    self.was_all_colors_achieved = true;
                    all_colors_score *= 2;
                }

                score += all_colors_score;

                let mut one_of_each_score = 0;
                if self.has_scanned_one_of_each_for(creature._type, drone_id) {
                    one_of_each_score += 4;
                }
                if !self.was_1_of_each_achieved {
                    self.was_1_of_each_achieved = true;
                    one_of_each_score *= 2;
                }

                score += one_of_each_score;

                if drone.is_mine {
                    self.my_scan_count += 1;
                    self.my_score += score;
                } else {
                    self.foe_scan_count += 1;
                    self.foe_score += score;
                }
            }
        }
    }

    fn find_best_move(&self) -> Option<Move> {
        let mut best_move: Option<Move> = None;
        let mut best_score: f64 = i32::min_value() as f64;

        let possible_moves = self.get_possible_moves();

        // Shuffle the possible moves to avoid always picking the same one when evaluation is equal
        let shuffled_possible_moves = {
            let mut rng = rand::thread_rng();
            let mut moves = possible_moves.clone();
            moves.shuffle(&mut rng);
            moves
        };

        for m in shuffled_possible_moves {
            let mut new_state = self.clone();
            new_state.apply_moves(m.clone());

            let score =
                new_state.minimax(3, i32::min_value() as f64, i32::max_value() as f64, true);
            if score > best_score {
                best_score = score;
                best_move = Some(m.clone());
            }
        }

        best_move
    }

    // should return true if all types of creatures for the provided colors have been scanned
    fn has_scanned_all_creatures_of_color_for(&self, color: i32, drone_id: i32) -> bool {
        let mut has_scanned_type_0 = false;
        let mut has_scanned_type_1 = false;
        let mut has_scanned_type_2 = false;

        for creature in self.creatures.values() {
            let was_scanned = self
                .scans
                .contains(&format!("{}:{}", drone_id, creature.id));

            if creature.color == color && was_scanned {
                if creature._type == 0 {
                    has_scanned_type_0 = true;
                } else if creature._type == 1 {
                    has_scanned_type_1 = true;
                } else if creature._type == 2 {
                    has_scanned_type_2 = true;
                }
            }

            if has_scanned_type_0 && has_scanned_type_1 && has_scanned_type_2 {
                return true;
            }
        }

        false
    }

    // should return true if all colors of creatures for the provided type have been scanned
    fn has_scanned_one_of_each_for(&self, _type: i32, drone_id: i32) -> bool {
        let mut has_scanned_color_0 = false;
        let mut has_scanned_color_1 = false;
        let mut has_scanned_color_2 = false;
        let mut has_scanned_color_3 = false;

        for creature in self.creatures.values() {
            let was_scanned = self
                .scans
                .contains(&format!("{}:{}", drone_id, creature.id));

            if creature._type == _type && was_scanned {
                if creature.color == 0 {
                    has_scanned_color_0 = true;
                } else if creature.color == 1 {
                    has_scanned_color_1 = true;
                } else if creature.color == 2 {
                    has_scanned_color_2 = true;
                } else if creature.color == 3 {
                    has_scanned_color_3 = true;
                }
            }

            if has_scanned_color_0
                && has_scanned_color_1
                && has_scanned_color_2
                && has_scanned_color_3
            {
                return true;
            }
        }

        false
    }
}

/**
 * Score points by scanning valuable fish faster than your opponent.
 **/
fn main() {
    let mut game_state = GameState::new();

    loop {
        game_state.update_state();

        for _i in 0..game_state.my_drone_count as usize {
            // Write an action using println!("message...");
            // To debug: eprintln!("Debug message...");

            let m = game_state.find_best_move();

            if m.is_none() {
                println!("WAIT 0");
                continue;
            } else {
                let m = m.unwrap();
                let light = if m.light { "1" } else { "0" };

                if m.should_move {
                    println!("MOVE {} {} {}", m.x.unwrap(), m.y.unwrap(), light);
                } else {
                    println!("WAIT {}", light);
                }
            }
        }
    }
}
