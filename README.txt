into_service() и as_service() нужен чаще всего для тестирования, его нужно использовать для Router<_>, example:
let s = Router::new().route("/", get(|| async move {}));
s.as_service() or s.into_service()


response Extensions могут быть использованы для передачи extensions в middleware