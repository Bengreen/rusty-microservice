use warp::Filter;

mod filters {
    use warp::Filter;
    use super::handlers;

    pub fn sample(
        basepath: &'static str,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path(basepath)
        .and(
            sample_1()
        )
    }

    pub fn sample_1() -> impl Filter<Extract = (impl warp::Reply, ), Error = warp::Rejection> + Clone {
        warp::get()
            .and(warp::path!("sample1"))
            .and_then(handlers::sample_h)
    }
}

mod handlers {
    use std::convert::Infallible;

    pub async fn sample_h() -> Result<impl warp::Reply, Infallible> {
        println!("Sample:");
        Ok("Sample")
    }
}

pub async fn sample_listen<'a>(
    basepath: &'static str,
    port: u16,
) {
    println!("Starting sample service http on {}", port);

    let api = filters::sample(basepath);

    let routes = api.with(warp::log("sample"));

    println!("Starting sample service");
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}


