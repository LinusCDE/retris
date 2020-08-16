use fxhash::FxHashMap;
use libremarkable::input::multitouch::MultitouchEvent;
use libremarkable::cgmath::Point2;

pub const SWIPE_DIRECTION_CHUNK_DIST: u16 = 25; // px
/// Min TrackedFinger.swipe_chunk_count to cound as completed swipe
pub const SWIPE_DIRECTION_COMPLETED_MIN_CHUNK_COUNT: u16 = 2;
/// The higher this value, the stricter the direction of the 
/// gesture has to be.
/// Is calculated on a per GESTURE_DIRECTION_CHUNK_DIST basis
/// by the following formula: abs(intended_direction_dist) / abs(orthogonal_direction_dist)
/// For the gesture "Up" that could be e.g: abs(-100) / abs(49) => 2.04#
/// The value 2.0 basically allows for NORTH, EAST, WEST, SOUTH but not
/// NORTH_EAST, NORTH_WEST, SOUTH_EAST, SOUTH_WEST if using a compass
/// as analogy for the angles.
pub const SWIPE_DIRECTION_MIN_RATIO: f32 = 1.5;

/// Allow a new gesture to register, even if the current
/// swipe was messed up.
pub const SWIPE_ALLOW_MULTIPLE_SWIPES_AT_ONCE: bool = true;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Trigger {
    /// The swipe gesture will fire, when the user
    /// releases his finger.
    Completed,
    /// The swipe gesture will fire, when the
    /// given distance is exeeded. After that
    /// a new swipe could be recognized
    /// immediately again.
    /// The distance is detected on a basis of
    /// SWIPE_DIRECTION_CHUNK_DIST.
    MinDistance(u16),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Swipe {
    pub direction: Direction,
    pub trigger: Trigger,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction { Up, Down, Left, Right }

struct TrackedFinger {
    swipe_chunk_count: u16,
    last_pos: Point2<u16>,
    current_pos: Point2<u16>,
    pub invalidated: bool, // If true, direction should be None!
    pub direction: Option<Direction>,
}

impl TrackedFinger {
    pub fn new(start_pos: Point2<u16>) -> Self{
        Self {
            swipe_chunk_count: 0,
            last_pos: start_pos,
            direction: None,
            current_pos: start_pos,
            invalidated: false,
        }
    }
    fn highest_orthogonal_dist(&self) -> u16 {
        let x_dist = (self.current_pos.x as i16 - self.last_pos.x as i16).abs() as u16;
        let y_dist = (self.current_pos.y as i16 - self.last_pos.y as i16).abs() as u16;
        return std::cmp::max(x_dist, y_dist)
    }

    fn update(&mut self, current_pos: Point2<u16>) {
        self.current_pos = current_pos;
        if self.highest_orthogonal_dist() < SWIPE_DIRECTION_CHUNK_DIST {
            return;
        }
        if self.invalidated && !SWIPE_ALLOW_MULTIPLE_SWIPES_AT_ONCE {
            // The direction changed. This gesture should not
            // be recognized as anything valid anymore.
            // It should not contain any direction.
            return;
        }

        let x_dist = self.current_pos.x as i16 - self.last_pos.x as i16;
        let y_dist = self.current_pos.y as i16 - self.last_pos.y as i16;

        let current_dir = if x_dist > y_dist.abs() && x_dist.abs() as f32 / y_dist.abs().max(1) as f32 > SWIPE_DIRECTION_MIN_RATIO {
            Some(Direction::Right)
        }else if -x_dist > y_dist.abs() && x_dist.abs() as f32 / y_dist.abs().max(1) as f32 > SWIPE_DIRECTION_MIN_RATIO {
            Some(Direction::Left)
        }else if y_dist > x_dist.abs() && y_dist.abs() as f32 / x_dist.abs().max(1) as f32 > SWIPE_DIRECTION_MIN_RATIO {
            Some(Direction::Down)
        }else if -y_dist > x_dist.abs() && y_dist.abs() as f32 / x_dist.abs().max(1) as f32 > SWIPE_DIRECTION_MIN_RATIO {
            Some(Direction::Up)
        }else {
            None
        };

        if current_dir.is_none() {
            self.invalidated = true;
            self.direction = None;
            return;
        }

        let current_dir = current_dir.unwrap();


        if let Some(expected_dir) = self.direction {
            if expected_dir != current_dir {
                // Previously detected direction changed
                self.invalidated = true;
                self.direction = None;
            }else {
                // Direction stayed the same
                self.last_pos = current_pos;
                self.swipe_chunk_count += 1;
            }
        }else {
            // This direction is the initial one and expected
            // to last till the gesture is announced.
            self.last_pos = self.current_pos;
            self.direction = Some(current_dir);
        }
    }
}

pub struct SwipeTracker {
    trackings: FxHashMap<i32 /* Tracking id */, TrackedFinger>,
}

impl SwipeTracker {
    pub fn new() -> Self {
        Self { trackings: Default::default() }
    }

    pub fn detect<'a>(&mut self, event: MultitouchEvent, conditions: &'a[Swipe]) -> Option<&'a Swipe> {
        match event {
            MultitouchEvent::Press { finger } => {
                self.trackings.insert(finger.tracking_id, TrackedFinger::new(finger.pos));
            },
            MultitouchEvent::Move { finger } => {
                let mut potential_swipe: Option<&'a Swipe> = None;
                self.trackings.entry(finger.tracking_id).and_modify(|tracked_finger|
                    tracked_finger.update(finger.pos)
                ).and_modify(|tracked_finger| {
                    if let Some(intermediate_direction) = tracked_finger.direction {
                        for swipe in conditions.iter() {
                            if swipe.direction == intermediate_direction {
                                if let Trigger::MinDistance(min_dist) = swipe.trigger {
                                    if min_dist <= SWIPE_DIRECTION_CHUNK_DIST * tracked_finger.swipe_chunk_count {
                                        // Reset swipe info
                                        *tracked_finger = TrackedFinger::new(tracked_finger.last_pos);
                                        // Swipe found
                                        potential_swipe = Some(swipe);
                                    }
                                }
                            }
                        }
                    }
                });

                if potential_swipe.is_some() {
                    return potential_swipe;
                }
            },
            MultitouchEvent::Release { finger } => {
                if let Some(tracked_finger) = self.trackings.remove(&finger.tracking_id) {
                    if tracked_finger.swipe_chunk_count >= SWIPE_DIRECTION_COMPLETED_MIN_CHUNK_COUNT {
                        if let Some(completed_direction) = tracked_finger.direction {
                            for swipe in conditions.iter() {
                                if swipe.direction == completed_direction && swipe.trigger == Trigger::Completed {
                                    // Completed swipe detected
                                    return Some(swipe);
                                }
                            }
                        }
                    }
                }
                self.trackings.remove(&finger.tracking_id);
            },
            _ => { }
        }

        None
    }
}