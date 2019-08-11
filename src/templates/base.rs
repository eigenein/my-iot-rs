//! Page base.

markup::define! {
    Base(body: Box<std::fmt::Display>) {
        {markup::doctype()}
        html[lang = "en"] {
            head {
                title { "My IoT" }
                meta[charset = "utf-8"];
                meta[name = "viewport", content = "width=device-width, initial-scale=1"];
                meta["http-equiv" = "refresh", content = "60"];
                link[rel = "apple-touch-icon", sizes="180x180", href="/apple-touch-icon.png"];
                link[rel = "icon", type="image/png", sizes="32x32", href="/favicon-32x32.png"];
                link[rel = "icon", type="image/png", sizes="16x16", href="/favicon-16x16.png"];
                link[rel = "stylesheet", href = "https://cdnjs.cloudflare.com/ajax/libs/bulma/0.7.4/css/bulma.min.css"];
                script[src = "https://kit.fontawesome.com/e88ef3598d.js"] {}
                script[src = "https://cdn.plot.ly/plotly-1.5.0.min.js"] {}
                style { ".reading { height: 100% }" }
            }
            body {
                {markup::raw(body)}
                footer.footer {
                    div.container {
                        div.columns {
                            div.column."is-4" {
                                p {
                                    i.fas."fa-circle"."has-text-info" {} " "
                                    a[href = "https://github.com/eigenein/my-iot-rs"] {
                                        strong { "My IoT " {clap::crate_version!()} }
                                    }
                                    " by " a[href = "https://github.com/eigenein"] { strong { "eigenein" } }
                                }
                                p {
                                    i.fas."fa-certificate"."has-text-primary" {} " "
                                    "Made with " a[href = "https://bulma.io/"] { strong { "Bulma" } }
                                }
                                p {
                                    i.fab."fa-fort-awesome"."has-text-success" {} " "
                                    "Icons by " a[href = "https://fontawesome.com/"] { strong { "Font Awesome" } }
                                }
                            }
                        }
                    }
                }
                script {
                    {markup::raw(r#"
                        document.addEventListener('DOMContentLoaded', () => {
                            const $navbarBurgers = Array.prototype.slice.call(document.querySelectorAll('.navbar-burger'), 0);
                            if ($navbarBurgers.length > 0) {
                                $navbarBurgers.forEach(el => {
                                    el.addEventListener('click', () => {
                                        const $target = document.getElementById(el.dataset.target);
                                        el.classList.toggle('is-active');
                                        $target.classList.toggle('is-active');
                                    });
                                });
                            }
                        });
                    "#)}
                }
            }
        }
    }
}
