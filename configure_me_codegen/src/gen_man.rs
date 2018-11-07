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

fn generate_params(man: Manual, config: &Config) -> Manual {
    config
        .params
        .iter()
        .filter(|param| param.argument).map(|param| {
            let opt = Opt::new(&param.name.to_uppercase()).long(&::codegen::param_long(param));
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
            let flag = if let Some(doc) = &switch.doc {
                flag.help(&doc)
            } else {
                flag
            };
            flag
        })
        .fold(man, |man, flag| man.flag(flag))
}

pub fn generate_man_page(config: &Config) -> String {
    let man = generate_meta(config);
    let man = if let Some(doc) = &config.general.doc {
        man.description(doc.to_owned())
    } else {
        man
    };
    let man = generate_params(man, config);
    let man = generate_switches(man, config);

    man.render()
}
