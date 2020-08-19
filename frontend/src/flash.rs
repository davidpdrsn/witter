use seed::prelude::Orders;
use crate::Msg;

#[derive(Debug, Default)]
pub struct Flash {
    msg: Option<FlashMsg>,
}

impl Flash {
    pub fn set_notice(&mut self, notice: &str, orders: &mut impl Orders<Msg>) {
        self.msg = Some(FlashMsg::Notice(notice.to_string()));
        self.set_clearing_time(orders);
    }

    pub fn set_error(&mut self, error: &str, orders: &mut impl Orders<Msg>) {
        self.msg = Some(FlashMsg::Error(error.to_string()));
        self.set_clearing_time(orders);
    }

    pub fn clear(&mut self) {
        self.msg = None;
    }

    pub fn get(&self) -> Option<&FlashMsg> {
        self.msg.as_ref()
    }

    fn set_clearing_time(&self, orders: &mut impl Orders<Msg>) {
        orders.perform_cmd(seed::app::cmds::timeout(5000, || Msg::ClearFlash));
    }
}

#[derive(Debug)]
pub enum FlashMsg {
    Notice(String),
    Error(String),
}
