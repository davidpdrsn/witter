use async_std::sync::Mutex;
use chrono::prelude::*;
use lazy_static::lazy_static;

#[cfg(test)]
use std::future::Future;

#[cfg(test)]
pub async fn freeze_time<T, F, Fut>(time: DateTime<Utc>, mut f: F) -> Fut::Output
where
    F: FnMut() -> Fut,
    Fut: Future,
{
    assert!(FROZEN_TIME.lock().await.is_none());

    *FROZEN_TIME.lock().await = Some(time);
    let result = f().await;
    *FROZEN_TIME.lock().await = None;
    result
}

lazy_static! {
    static ref FROZEN_TIME: Mutex<Option<DateTime<Utc>>> = Mutex::new(None);
}

pub async fn current_time() -> DateTime<Utc> {
    if let Some(time) = *FROZEN_TIME.lock().await {
        time
    } else {
        Utc::now()
    }
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    #[async_std::test]
    async fn freezing_time() {
        let time = Utc.ymd(1970, 1, 1).and_hms(0, 1, 1);

        freeze_time::<(), _, _>(time, || async {
            assert_eq!(current_time().await, time);
        })
        .await;

        assert_ne!(current_time().await, time);
    }

    #[async_std::test]
    #[should_panic]
    async fn cannot_nest_freezing_time() {
        let time = Utc.ymd(1970, 1, 1).and_hms(0, 1, 1);

        freeze_time::<(), _, _>(time, || async {
            freeze_time::<(), _, _>(time, || async {
                assert_eq!(current_time().await, time);
            })
            .await;
        })
        .await;
    }
}
