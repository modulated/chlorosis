#[macro_export]
macro_rules! increment_register {
    ($dev:ident, $reg:expr) => {
        $dev.cpu.check_half_carry_add_byte($reg, Byte(1));
        $reg += 1;
        $dev.cpu.n_flag = false;
        $dev.cpu.check_zero($reg);
    };
}

#[macro_export]
macro_rules! decrement_register {
    ($dev:ident, $reg:expr) => {
        $dev.cpu.check_half_carry_sub_byte($reg, Byte(1));
        $reg -= 1;
        $dev.cpu.n_flag = true;
        $dev.cpu.check_zero($reg);
    };
}

#[macro_export]
macro_rules! addition_register_pairs {
    ($dev:ident, $a:expr, $b:expr, $write:expr) => {
        $dev.cpu.check_half_carry_add_address($a, $b);
        $dev.cpu.check_carry_add_address($a, $b);
        $write($a + $b);
        $dev.cpu.n_flag = false;
    };
}
