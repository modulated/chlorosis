#[macro_export]
macro_rules! increment_register {
    ($cpu:ident, $reg:expr) => {
        $cpu.check_half_carry_add_byte($reg, Byte(1));
        $reg += 1;
        $cpu.n_flag = false;
        $cpu.check_zero($reg);
        $cpu.cost = 1;
    };
}

#[macro_export]
macro_rules! decrement_register {
    ($cpu:ident, $reg:expr) => {
        $cpu.check_half_carry_sub_byte($reg, Byte(1));
        $reg -= 1;
        $cpu.n_flag = true;
        $cpu.check_zero($reg);
        $cpu.cost = 1;
    };
}

#[macro_export]
macro_rules! addition_register_pairs {
    ($cpu:ident, $a:expr, $b:expr, $write:expr) => {
        $cpu.check_half_carry_add_address($a, $b);
        $cpu.check_carry_add_address($a, $b);
        $write($a + $b);
        $cpu.n_flag = false;
        $cpu.cost = 2;
    };
}
