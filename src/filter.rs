pub enum Filter {
    HFlip,
    Rotate {
        angle: String
    },
    Transpose {
        dir: TransposeDir,
        passthrough: TransposePassthrough
    },
    VFlip
}

pub enum TransposeDir {
    CClockFlip,
    Clock,
    CClock,
    ClockFlip
}

pub enum TransposePassthrough {
    None,
    Portrait,
    Landscape
}