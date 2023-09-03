use std::sync::mpsc;

type T<'a> = Box<dyn Send + 'a>;

pub struct Unit<T> {
    id: usize,
    pub sender: mpsc::Sender<T>,
    pub receiver: mpsc::Receiver<T>,
}

pub struct Commander<T> {
    pub senders: Vec<mpsc::Sender<T>>,
    pub receiver: mpsc::Receiver<T>,
}

pub struct Army<T> {
    pub units: Vec<Option<Unit<T>>>,

    pub commander: Commander<T>,
}

impl Unit<T<'_>> {
    pub fn build<'a>(
        id: usize,
        sender: mpsc::Sender<T<'a>>,
        receiver: mpsc::Receiver<T<'a>>,
    ) -> Unit<T<'a>> {
        Unit {
            id,
            sender,
            receiver,
        }
    }
}

impl Commander<T<'_>> {
    pub fn build<'a>(
        senders: Vec<mpsc::Sender<T<'a>>>,
        receiver: mpsc::Receiver<T<'a>>,
    ) -> Commander<T<'a>> {
        Commander { senders, receiver }
    }
}

impl Army<T<'_>> {
    pub fn build<'a>(size: usize) -> Army<T<'a>> {
        let mut units = Vec::with_capacity(size);

        let mut senders = Vec::with_capacity(size);

        let (unit_to_commander_sender, unit_to_commander_receiver) = mpsc::channel();

        for idx in 0..size {
            let (commander_to_unit_sender, commander_to_unit_receiver) = mpsc::channel();
            let sender = unit_to_commander_sender.clone();
            let receiver = commander_to_unit_receiver;

            units.push(Some(Unit::build(idx, sender, receiver)));
            senders.push(commander_to_unit_sender);
        }

        let commander = Commander::build(senders, unit_to_commander_receiver);

        Army { units, commander }
    }
}
