// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of
// the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/

//! This module contains extra settling functionality for `frames::Frame`.

// -------------------------------------------------------------------------------------------------

use qualia::{Area, Position, Vector, SurfaceAccess, SurfaceId};

use frame::{Frame, Geometry, Mode, Side};
use searching::Searching;
use packing::Packing;

// -------------------------------------------------------------------------------------------------

/// Extension trait for `Frame` adding more settling functionality.
pub trait Settling {
    /// Settle self in buildable of target and relax it.
    ///
    /// If `area` is provided settle the surface as floating with given position and size.
    fn settle(&mut self, target: &mut Frame, area: Option<Area>, sa: &mut SurfaceAccess);

    /// Remove given frame, relax old parent and settle the frame on given target.
    fn resettle(&mut self, target: &mut Frame, sa: &mut SurfaceAccess);

    /// Pop the surface `pop` and its parents inside surface `self`.
    ///
    /// After calling this function `pop` will be most recently used frame inside `self`.
    fn pop_recursively(&mut self, pop: &mut Frame);

    /// Changes frames geometry and resizes all subframe accordingly.
    fn change_geometry(&mut self, geometry: Geometry, sa: &mut SurfaceAccess);

    /// Adds another container into given place in frame layout if needed.
    ///
    /// This method is used when jumping into leaf frame to create container to handle the leaf
    /// and jumped frame.
    ///
    /// Returns
    ///  - `self` if it is container with one child,
    ///  - parent if parent has one child
    ///  - newly created container frame otherwise
    fn ramify(&mut self, geometry: Geometry) -> Frame;

    /// Removes unnecessary layers of container frames containing only one container or leaf frame.
    fn deramify(&mut self);

    /// Places frame `self` on given `side` of `target` frame.
    fn jumpin(&mut self, side: Side, target: &mut Frame, sa: &mut SurfaceAccess);

    /// Removes frame `self` from frame layout and then places it using `jumpin` method.
    fn jump(&mut self, side: Side, target: &mut Frame, sa: &mut SurfaceAccess);

    /// Anchorizes floating frame.
    fn anchorize(&mut self, sa: &mut SurfaceAccess);

    /// Deanchorizes frame. Floating frame must be attached to workspace so it will be resettled if
    /// necessary.
    fn deanchorize(&mut self, area: Area, sa: &mut SurfaceAccess);

    /// Set new position for given frame and move it subframes accordingly.
    fn set_position(&mut self, pos: Position);

    /// Move the frame and all subframes by given vector.
    fn move_with_contents(&mut self, vector: Vector);

    /// Removes frame `self`, relaxes old parent and destroys the frame.
    fn destroy_self(&mut self, sa: &mut SurfaceAccess);
}

// -------------------------------------------------------------------------------------------------

impl Settling for Frame {
    fn settle(&mut self, target: &mut Frame, area: Option<Area>, sa: &mut SurfaceAccess) {
        if let Some(ref mut buildable) = target.find_buildable() {
            if buildable.get_geometry() == Geometry::Stacked {
                buildable.prepend(self);
                if let Some(area) = area {
                    self.set_size(area.size, sa);
                    self.set_position(area.pos);
                    self.set_plumbing_is_anchored(false);
                } else {
                    self.set_plumbing_is_anchored(true);
                }
            } else {
                buildable.append(self);
                self.set_plumbing_is_anchored(true);
            }
            buildable.relax(sa);
        }
    }

    fn resettle(&mut self, target: &mut Frame, sa: &mut SurfaceAccess) {
        self.remove_self(sa);
        self.settle(target, None, sa);
    }

    fn pop_recursively(&mut self, pop: &mut Frame) {
        // If we reached `self` we can finish
        if self.equals_exact(pop) {
            return;
        }

        // If there's nothing above we can finish
        if let Some(ref mut parent) = pop.get_parent() {
            // If it is `stacked` frame we have to pop it also spatially
            if parent.get_geometry() == Geometry::Stacked {
                pop.remove();
                parent.prepend(pop);
            }

            // Pop in temporal order
            pop.pop();

            // Do the same recursively on trunk
            self.pop_recursively(parent);
        }
    }

    fn change_geometry(&mut self, geometry: Geometry, sa: &mut SurfaceAccess) {
        self.set_plumbing_geometry(geometry);
        self.homogenize(sa);
    }

    fn ramify(&mut self, geometry: Geometry) -> Frame {
        let parent = self.get_parent().expect("should have parent");
        if self.count_children() == 1 {
            return self.clone();
        }
        if parent.count_children() == 1 {
            return parent;
        }

        let distancer_mode = if self.get_mode().is_top() {
            self.get_mode()
        } else {
            Mode::Container
        };

        let frame_mode = if self.get_mode().is_leaf() {
            self.get_mode()
        } else {
            Mode::Container
        };

        let mut distancer = Frame::new(SurfaceId::invalid(),
                                       distancer_mode,
                                       geometry,
                                       self.get_position(),
                                       self.get_size(),
                                       self.get_title(),
                                       true);
        self.prejoin(&mut distancer);
        self.remove();
        self.set_plumbing_mode(frame_mode);
        let opposite = self.get_position().opposite();
        self.move_with_contents(opposite);
        distancer.prepend(self);
        distancer
    }

    fn deramify(&mut self) {
        let len = self.count_children();
        if len == 1 {
            let mut first = self.get_first_time().expect("should have exactly one child");
            let len = first.count_children();
            if len == 1 {
                let mut second = first.get_first_time().expect("should have exactly one child");
                first.remove();
                second.remove();
                self.prepend(&mut second);
                first.destroy();
            } else if len == 0 {
                self.set_plumbing_mode(first.get_mode());
                self.set_plumbing_sid(first.get_sid());
                first.remove();
                first.destroy();
            }
        }
    }

    fn jumpin(&mut self, side: Side, target: &mut Frame, sa: &mut SurfaceAccess) {
        if let Some(mut target_parent) = target.get_parent() {
            match side {
                Side::Before => {
                    target.prejoin(self);
                    target_parent.relax(sa);
                }
                Side::After => {
                    target.adjoin(self);
                    target_parent.relax(sa);
                }
                Side::On => {
                    let mut new_target = {
                        if !target_parent.get_mode().is_top() &&
                            target_parent.count_children() == 1 {
                            target_parent.clone()
                        } else if target.get_mode().is_leaf() {
                            target.ramify(Geometry::Stacked)
                        } else {
                            target.clone()
                        }
                    };

                    self.settle(&mut new_target, None, sa);
                }
            }
        }
    }

    fn jump(&mut self, side: Side, target: &mut Frame, sa: &mut SurfaceAccess) {
        self.remove_self(sa);
        self.jumpin(side, target, sa);
    }

    fn anchorize(&mut self, sa: &mut SurfaceAccess) {
        if self.get_mode().is_reanchorizable() && !self.is_anchored() {
            // NOTE: Floating surface must be direct child of workspace.
            let parent = self.get_parent().expect("should have parent");
            self.set_size(parent.get_size(), sa);
            self.set_position(Position::default());
            self.set_plumbing_is_anchored(true);
        }
    }

    fn deanchorize(&mut self, area: Area, sa: &mut SurfaceAccess) {
        if self.get_mode().is_reanchorizable() && self.is_anchored() {
            let mut workspace = self.find_top().expect("should have toplevel");
            if workspace.get_mode().is_workspace() {
                let parent = self.get_parent().expect("should have parent");
                if !parent.equals_exact(&workspace) {
                    self.remove_self(sa);
                    workspace.prepend(self);
                }
                self.set_size(area.size, sa);
                self.set_position(area.pos);
                self.set_plumbing_is_anchored(false);
            }
        }
    }

    fn set_position(&mut self, pos: Position) {
        let vector = pos - self.get_position();
        self.move_with_contents(vector);
    }

    fn move_with_contents(&mut self, vector: Vector) {
        // Update frames position
        let new_position = self.get_position() + vector.clone();
        self.set_plumbing_position(new_position);

        // Move all subframes
        for mut frame in self.space_iter() {
            frame.move_with_contents(vector.clone());
        }
    }

    fn destroy_self(&mut self, sa: &mut SurfaceAccess) {
        self.remove_self(sa);
        self.destroy();
    }
}

// -------------------------------------------------------------------------------------------------
