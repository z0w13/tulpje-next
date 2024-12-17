use redlight::config::Ignore;

pub struct Config;

impl redlight::config::CacheConfig for Config {
    type Channel<'a> = Ignore;
    type CurrentUser<'a> = Ignore;
    type Emoji<'a> = Ignore;
    type Guild<'a> = Ignore;
    type Integration<'a> = Ignore;
    type Member<'a> = Ignore;
    type Message<'a> = Ignore;
    type Presence<'a> = Ignore;
    type Role<'a> = Ignore;
    type StageInstance<'a> = Ignore;
    type Sticker<'a> = Ignore;
    type User<'a> = super::user::CachedUser;
    type VoiceState<'a> = Ignore;
}
