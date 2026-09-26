use std::{cell::RefCell, rc::Rc};

use anyhow::{anyhow, Result};
use msfs::commbus::CommBus;

use super::sentry_gauge::{wrap_gauge_with_sentry, SentryGauge};
use crate::interface::FunctionQueue;

/// The main state for the interface
struct NavigationDataInterface<'a> {
    _commbus: CommBus<'a>,
    function_queue: Rc<RefCell<FunctionQueue>>,
}

impl SentryGauge for NavigationDataInterface<'_> {
    fn initialize() -> Result<Self>
    where
        Self: Sized,
    {
        // Initialize commbus
        let mut commbus = CommBus::default();
        let function_queue = Rc::new(RefCell::new(FunctionQueue::default()));

        // Create the NAVIGRAPH_CallFunction callback
        let function_queue_clone = Rc::clone(&function_queue);
        commbus
            .register("NAVIGRAPH_CallFunction", move |args| {
                // Try to get the queue
                let Ok(mut function_queue) = function_queue_clone.try_borrow_mut() else {
                    sentry::capture_message(
                        "Unable to borrow processing queue",
                        sentry::Level::Warning,
                    );
                    return;
                };

                function_queue.push(args);
            })
            .ok_or(anyhow!("Unable to register NAVIGRAPH_CallFunction"))?;

        // Send first heartbeat
        function_queue.try_borrow_mut()?.heartbeat()?;

        Ok(Self {
            _commbus: commbus,
            function_queue,
        })
    }

    fn update(&mut self) -> Result<()> {
        self.function_queue.try_borrow_mut()?.update()
    }
}

crate::sentry_gauge!(NavigationDataInterface, navigation_data_interface);
