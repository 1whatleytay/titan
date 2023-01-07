pub trait SyscallCodeProvider {
    fn code(&self) -> u32;
}

pub trait SyscallHandler<T> {
    async fn print_integer(&mut self, state: &mut T);
    async fn print_float(&mut self, state: &mut T);
    async fn print_double(&mut self, state: &mut T);
    async fn print_string(&mut self, state: &mut T);
    async fn read_integer(&mut self, state: &mut T);
    async fn read_float(&mut self, state: &mut T);
    async fn read_double(&mut self, state: &mut T);
    async fn read_string(&mut self, state: &mut T);
    async fn alloc_heap(&mut self, state: &mut T);
    async fn terminate(&mut self, state: &mut T);
    async fn print_character(&mut self, state: &mut T);
    async fn read_character(&mut self, state: &mut T);
    async fn open_file(&mut self, state: &mut T);
    async fn read_file(&mut self, state: &mut T);
    async fn write_file(&mut self, state: &mut T);
    async fn close_file(&mut self, state: &mut T);
    async fn terminate_valued(&mut self, state: &mut T);
    async fn system_time(&mut self, state: &mut T);
    async fn midi_out(&mut self, state: &mut T);
    async fn sleep(&mut self, state: &mut T);
    async fn midi_out_sync(&mut self, state: &mut T);
    async fn print_hexadecimal(&mut self, state: &mut T);
    async fn print_binary(&mut self, state: &mut T);
    async fn print_unsinged(&mut self, state: &mut T);
    async fn set_seed(&mut self, state: &mut T);
    async fn random_int(&mut self, state: &mut T);
    async fn random_int_ranged(&mut self, state: &mut T);
    async fn random_float(&mut self, state: &mut T);
    async fn random_double(&mut self, state: &mut T);

    // Dialogs are not supported at the moment.
    // That would probably be another trait. PRs welcome.

    // code: $v0
    async fn dispatch(&mut self, state: &mut T, code: u32) {
        match code {
            1 => self.print_integer(state),
            2 => self.print_float(state),
            3 => self.print_double(state),
            4 => self.print_string(state),
            5 => self.read_integer(state),
            6 => self.read_float(state),
            7 => self.read_double(state),
            8 => self.read_string(state),
            9 => self.alloc_heap(state),
            10 => self.terminate(state),
            11 => self.print_character(state),
            12 => self.read_character(state),
            13 => self.open_file(state),
            14 => self.read_file(state),
            15 => self.write_file(state),
            16 => self.close_file(state),
            17 => self.terminate_valued(state),
            30 => self.system_time(state),
            31 => self.midi_out(state),
            32 => self.sleep(state),
            33 => self.midi_out_sync(state),
            34 => self.print_hexadecimal(state),
            35 => self.print_binary(state),
            36 => self.print_unsinged(state),
            40 => self.set_seed(state),
            41 => self.random_int(state),
            42 => self.random_int_ranged(state),
            43 => self.random_float(state),
            44 => self.random_double(state),
        }
    }
}

pub trait SyscallHandlerDefault<T: SyscallCodeProvider>: SyscallHandler<T> {
    fn dispatch(&mut self, state: &mut T) {
        self.dispatch(state, state.code())
    }
}
