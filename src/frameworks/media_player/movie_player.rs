/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `MPMoviePlayerController` etc.

use crate::dyld::{ConstantExports, HostConstant};
use crate::frameworks::foundation::{ns_string, ns_url, NSInteger};
use crate::frameworks::uikit::ui_device::UIDeviceOrientation;
use crate::objc::{id, msg, msg_class, objc_classes, release, retain, ClassExports};
use crate::Environment;
use std::collections::VecDeque;

#[derive(Default)]
pub struct State {
    active_player: Option<id>,
    /// Various apps (e.g. Crash Bandicoot Nitro Kart 3D and Spore Origins)
    /// create or start a player and await some kind of notification, but can't
    /// handle it if that notification happens immediately. This queue lets us
    /// delay such notifications until the app next returns to the run loop,
    /// which seems to be late enough.
    pending_notifications: VecDeque<(&'static str, id)>,
}
impl State {
    fn get(env: &mut Environment) -> &mut Self {
        &mut env.framework_state.media_player.movie_player
    }
}

type MPMovieScalingMode = NSInteger;

// Values might not be correct, but as these are linked symbol constants, it
// shouldn't matter.
pub const MPMoviePlayerPlaybackDidFinishNotification: &str =
    "MPMoviePlayerPlaybackDidFinishNotification";
/// Apparently an undocumented, private API. Spore Origins uses it.
pub const MPMoviePlayerContentPreloadDidFinishNotification: &str =
    "MPMoviePlayerContentPreloadDidFinishNotification";
// TODO: More notifications?
pub const UIKeyboardDidShowNotification: &str =
    "UIKeyboardDidShowNotification";
pub const UIKeyboardWillShowNotification: &str =
    "UIKeyboardWillShowNotification";
pub const UIKeyboardDidHideNotification: &str =
    "UIKeyboardDidHideNotification";
pub const UIKeyboardWillHideNotification: &str =
    "UIKeyboardWillHideNotification";
pub const UIDeviceOrientationDidChangeNotification: &str =
    "UIDeviceOrientationDidChangeNotification";
pub const UIApplicationLaunchOptionsRemoteNotificationKey: &str =
    "UIApplicationLaunchOptionsRemoteNotificationKey";

/// `NSNotificationName` values.
pub const CONSTANTS: ConstantExports = &[
    (
        "_MPMoviePlayerPlaybackDidFinishNotification",
        HostConstant::NSString(MPMoviePlayerPlaybackDidFinishNotification),
    ),
    (
        "_MPMoviePlayerContentPreloadDidFinishNotification",
        HostConstant::NSString(MPMoviePlayerContentPreloadDidFinishNotification),
    ),
    (
        "_UIKeyboardDidShowNotification",
        HostConstant::NSString(UIKeyboardDidShowNotification),
    ),
    (
        "_UIKeyboardWillShowNotification",
        HostConstant::NSString(UIKeyboardWillShowNotification),
    ),
    (
        "_UIKeyboardDidHideNotification",
        HostConstant::NSString(UIKeyboardDidHideNotification),
    ),
    (
        "_UIKeyboardWillHideNotification",
        HostConstant::NSString(UIKeyboardWillHideNotification),
    ),
    (
        "_UIDeviceOrientationDidChangeNotification",
        HostConstant::NSString(UIDeviceOrientationDidChangeNotification),
    ),
    (
        "_UIApplicationLaunchOptionsRemoteNotificationKey",
        HostConstant::NSString(UIApplicationLaunchOptionsRemoteNotificationKey),
    ),
];

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation MPMoviePlayerController: NSObject

// TODO: actual playback

- (id)initWithContentURL:(id)url { // NSURL*
    log!(
        "TODO: [(MPMoviePlayerController*){:?} initWithContentURL:{:?} ({:?})]",
        this,
        url,
        ns_url::to_rust_path(env, url),
    );

    // Act as if loading immediately completed (Spore Origins waits for this).
    State::get(env).pending_notifications.push_back(
        (MPMoviePlayerContentPreloadDidFinishNotification, this)
    );

    this
}

- (())setScalingMode:(MPMovieScalingMode)_mode {
    // TODO
}
- (())setBackgroundColor:(id)_color {
    // TODO
}
- (())setOrientation:(NSInteger)_orient
            animated:(bool)_animated {
    // TODO why this is even called here?
}

// Apparently an undocumented, private API, but Spore Origins uses it.
- (())setMovieControlMode:(NSInteger)_mode {
    // Game-specific hack :(
    // Spore Origins subscribes to the playback finished notification 0.2s after
    // starting playback, so it misses the notification we send. When it
    // subscribes, it also calls this method, so this is an opportunity to send
    // the notification again.
    if env.bundle.bundle_identifier().starts_with("com.ea.spore") {
        log!("Applying game-specific hack for Spore Origins: sending MPMoviePlayerPlaybackDidFinishNotification again.");
        State::get(env).pending_notifications.push_back(
            (MPMoviePlayerPlaybackDidFinishNotification, this)
        );
    }
    // As this is undocumented and we don't have real video playback yet, let's
    // ignore it otherwise.
}

// Another undocumented one! But some apps may still use it :/
// https://stackoverflow.com/a/1390079/2241008
- (())setOrientation:(UIDeviceOrientation)_orientation animated:(bool)_animated {

}
    
// MPMediaPlayback implementation
- (())play {
    log!("TODO: [(MPMoviePlayerController*){:?} play]", this);
    if let Some(old) = env.framework_state.media_player.movie_player.active_player {
        let _: () = msg![env; old stop];
    }
    assert!(env.framework_state.media_player.movie_player.active_player.is_none());
    // Movie player is retained by the runtime until it is stopped
    retain(env, this);
    env.framework_state.media_player.movie_player.active_player = Some(this);

    // Act as if playback immediately completed (various apps wait for this).
    State::get(env).pending_notifications.push_back(
        (MPMoviePlayerPlaybackDidFinishNotification, this)
    );
}

- (())stop {
    log!("TODO: [(MPMoviePlayerController*){:?} stop]", this);
    assert!(this == env.framework_state.media_player.movie_player.active_player.take().unwrap());
    release(env, this);
}

@end

@implementation MPMediaQuery: NSObject
+ (id)playlistsQuery {
    crate::objc::nil
}
@end

};

/// For use by `NSRunLoop` via [super::handle_players]: check movie players'
/// status, send notifications if necessary.
pub(super) fn handle_players(env: &mut Environment) {
    while let Some(notif) = State::get(env).pending_notifications.pop_front() {
        let (name, object) = notif;
        let name = ns_string::get_static_str(env, name);
        let center: id = msg_class![env; NSNotificationCenter defaultCenter];
        // TODO: should there be some user info attached?
        let _: () = msg![env; center postNotificationName:name object:object];
    }
}
