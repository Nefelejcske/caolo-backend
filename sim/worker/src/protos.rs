pub mod cao_common {
    tonic::include_proto!("cao_common");
}

pub mod cao_script {
    tonic::include_proto!("cao_script");
}

pub mod cao_commands {
    tonic::include_proto!("cao_commands");
}

pub mod cao_world {
    tonic::include_proto!("cao_world");
}

pub mod cao_intents {
    tonic::include_proto!("cao_intents");
}

pub mod cao_users {
    tonic::include_proto!("cao_users");
}

impl From<caolo_sim::prelude::Axial> for cao_common::Axial {
    fn from(ax: caolo_sim::prelude::Axial) -> Self {
        cao_common::Axial { q: ax.q, r: ax.r }
    }
}
