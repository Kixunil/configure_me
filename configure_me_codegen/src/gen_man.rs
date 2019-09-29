use ::config::Config;
use ::man::prelude::*;

fn generate_meta(config: &Config) -> Manual {
    let man = if let Some(name) = &config.general.name {
        Manual::new(name)
    } else {
        Manual::new(&::build_helper::cargo::pkg::name())
    };

    let man = if let Some(summary) = &config.general.summary {
        man.about(&**summary)
    } else if let Some(summary) = ::build_helper::cargo::pkg::description() {
        man.about(summary)
    } else {
        man
    };

    let authors = ::build_helper::cargo::pkg::authors();
    authors.iter().fold(man, |man, author| {
        let mut name_email = author.split('<');
        if let Some(name) = name_email.next() {
            let author = Author::new(name.trim());

            let author = if let Some(email) = name_email.next() {
                let email = email.trim().trim_matches('>');
                author.email(email)
            } else {
                author
            };

            man.author(author)
        } else {
            man
        }
    })
}

fn generate_conf_file_param(man: Manual, config: &Config) -> Manual {
    if let Some(conf_file_param) = &config.general.conf_file_param {
        let opt = Opt::new("CONFIG_FILE").long(&::codegen::param_long_raw(conf_file_param.as_snake_case()));
        let opt = opt.help("Loads configuration from the specified CONFIG_FILE.");
        man.option(opt)
    } else {
        man
    }
}

fn generate_conf_dir_param(man: Manual, config: &Config) -> Manual {
    if let Some(conf_dir_param) = &config.general.conf_dir_param {
        let opt = Opt::new("CONFIG_DIR").long(&::codegen::param_long_raw(conf_dir_param.as_snake_case()));
        let opt = opt.help("Loads configuration from all files in the directory CONFIG_DIR.");
        man.option(opt)
    } else {
        man
    }
}

fn generate_params(man: Manual, config: &Config) -> Manual {
    config
        .params
        .iter()
        .filter(|param| param.argument).map(|param| {
            let opt = Opt::new(&param.name.as_upper_case().to_string()).long(&::codegen::param_long(param));
            let opt = if let Some(short) = ::codegen::param_short(param) {
                opt.short(&short)
            } else {
                opt
            };
            let opt = if let Some(doc) = &param.doc {
                opt.help(&doc)
            } else {
                opt
            };
            let opt = if let ::config::Optionality::DefaultValue(default) = &param.optionality {
                opt.default_value(&default)
            } else {
                opt
            };
            opt
        })
        .fold(man, |man, opt| man.option(opt))
}

fn generate_switches(man: Manual, config: &Config) -> Manual {
    config
        .switches
        .iter()
        .map(|switch| {
            let flag = Flag::new()
                .long(&::codegen::switch_long(switch));
            let flag = if let Some(short) = ::codegen::switch_short(switch) {
                flag.short(&short)
            } else {
                flag
            };
            let flag = if let Some(doc) = &switch.doc {
                flag.help(&doc)
            } else {
                flag
            };
            flag
        })
        .fold(man, |man, flag| man.flag(flag))
}

fn generate_param_env_vars(man: Manual, config: &Config) -> Manual {
    let prefix = config.general.env_prefix.as_ref().map_or_else(String::new, |prefix| [&prefix, "_"].join(""));
    config
        .params
        .iter()
        .filter(|param| param.env_var).map(|param| {
            let env = Env::new(&[&prefix as &str, &param.name.as_upper_case().to_string()].join(""));
            let env = if let Some(doc) = &param.doc {
                env.help(&doc)
            } else {
                env
            };
            let env = if let ::config::Optionality::DefaultValue(default) = &param.optionality {
                env.default_value(&default)
            } else {
                env
            };
            env
        })
        .fold(man, |man, env| man.env(env))
}

fn generate_switch_env_vars(man: Manual, config: &Config) -> Manual {
    let prefix = config.general.env_prefix.as_ref().map_or_else(String::new, |prefix| [&prefix, "_"].join(""));
    config
        .switches
        .iter()
        .filter(|switch| switch.env_var).map(|switch| {
            let env = Env::new(&[&prefix as &str, &switch.name.as_upper_case().to_string()].join(""));
            let env = if let Some(doc) = &switch.doc {
                env.help(&doc)
            } else {
                env
            };
            let env = if switch.is_inverted() {
                env.default_value("true")
            } else {
                env.default_value("false")
            };
            env
        })
        .fold(man, |man, env| man.env(env))
}

pub fn generate_man_page(config: &Config) -> String {
    let man = generate_meta(config);
    let man = if let Some(doc) = &config.general.doc {
        man.description(doc.to_owned())
    } else {
        man
    };
    let man = generate_conf_file_param(man, config);
    let man = generate_conf_dir_param(man, config);
    let man = generate_params(man, config);
    let man = generate_switches(man, config);
    let man = generate_param_env_vars(man, config);
    let man = generate_switch_env_vars(man, config);

    man.render()
}
