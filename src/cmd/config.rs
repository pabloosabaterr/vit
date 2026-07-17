use vit::die;
use vit::preference::{PREFERENCES_SET_DIMS, PREFERENCES_SET_MIN_FREQ, Preferences};

/*
 * #TODO: The idea with the flags is to leave written only what the user had set
 * manually.
 * Currently if you change dims, but you hadn't set min_freq, min_freq will be
 * written with its default value.
 */
pub fn config(ctx: &mut Preferences, args: &[String]) {
    if args.len() != 2 {
        die!("config expects exactly two arguments");
    }

    let flag = match args[0].as_str() {
        "dims" => PREFERENCES_SET_DIMS,
        "min-freq" => PREFERENCES_SET_MIN_FREQ,
        cfg => die!("unknown config: {}", cfg),
    };

    let val = match args[1].as_str() {
        "true" => 1,
        "false" => 0,
        val => val
            .parse::<usize>()
            .unwrap_or_else(|n| die!("{} is not a number", n)),
    };

    match flag {
        PREFERENCES_SET_DIMS => ctx.dims = val,
        PREFERENCES_SET_MIN_FREQ => ctx.min_freq = val,
        _ => unreachable!(),
    };

    ctx.write_preferences();
}
