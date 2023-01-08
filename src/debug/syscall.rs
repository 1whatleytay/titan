use async_trait::async_trait;
use crate::cpu::error::Error::CpuTrap;
use crate::cpu::error::Result;

#[async_trait]
pub trait SyscallHandler<T: Send> {
    async fn print_integer(&mut self, state: &mut T) -> Result<()>;
    async fn print_float(&mut self, state: &mut T) -> Result<()>;
    async fn print_double(&mut self, state: &mut T) -> Result<()>;
    async fn print_string(&mut self, state: &mut T) -> Result<()>;
    async fn read_integer(&mut self, state: &mut T) -> Result<()>;
    async fn read_float(&mut self, state: &mut T) -> Result<()>;
    async fn read_double(&mut self, state: &mut T) -> Result<()>;
    async fn read_string(&mut self, state: &mut T) -> Result<()>;
    async fn alloc_heap(&mut self, state: &mut T) -> Result<()>;
    async fn terminate(&mut self, state: &mut T) -> Result<()>;
    async fn print_character(&mut self, state: &mut T) -> Result<()>;
    async fn read_character(&mut self, state: &mut T) -> Result<()>;
    async fn open_file(&mut self, state: &mut T) -> Result<()>;
    async fn read_file(&mut self, state: &mut T) -> Result<()>;
    async fn write_file(&mut self, state: &mut T) -> Result<()>;
    async fn close_file(&mut self, state: &mut T) -> Result<()>;
    async fn terminate_valued(&mut self, state: &mut T) -> Result<()>;
    async fn system_time(&mut self, state: &mut T) -> Result<()>;
    async fn midi_out(&mut self, state: &mut T) -> Result<()>;
    async fn sleep(&mut self, state: &mut T) -> Result<()>;
    async fn midi_out_sync(&mut self, state: &mut T) -> Result<()>;
    async fn print_hexadecimal(&mut self, state: &mut T) -> Result<()>;
    async fn print_binary(&mut self, state: &mut T) -> Result<()>;
    async fn print_unsigned(&mut self, state: &mut T) -> Result<()>;
    async fn set_seed(&mut self, state: &mut T) -> Result<()>;
    async fn random_int(&mut self, state: &mut T) -> Result<()>;
    async fn random_int_ranged(&mut self, state: &mut T) -> Result<()>;
    async fn random_float(&mut self, state: &mut T) -> Result<()>;
    async fn random_double(&mut self, state: &mut T) -> Result<()>;

    // Dialogs are not supported at the moment.
    // That would probably be another trait. PRs welcome.

    // code: $v0
    async fn dispatch(&mut self, state: &mut T, code: u32) -> Result<()> {
        match code {
            1 => self.print_integer(state).await,
            2 => self.print_float(state).await,
            3 => self.print_double(state).await,
            4 => self.print_string(state).await,
            5 => self.read_integer(state).await,
            6 => self.read_float(state).await,
            7 => self.read_double(state).await,
            8 => self.read_string(state).await,
            9 => self.alloc_heap(state).await,
            10 => self.terminate(state).await,
            11 => self.print_character(state).await,
            12 => self.read_character(state).await,
            13 => self.open_file(state).await,
            14 => self.read_file(state).await,
            15 => self.write_file(state).await,
            16 => self.close_file(state).await,
            17 => self.terminate_valued(state).await,
            30 => self.system_time(state).await,
            31 => self.midi_out(state).await,
            32 => self.sleep(state).await,
            33 => self.midi_out_sync(state).await,
            34 => self.print_hexadecimal(state).await,
            35 => self.print_binary(state).await,
            36 => self.print_unsigned(state).await,
            40 => self.set_seed(state).await,
            41 => self.random_int(state).await,
            42 => self.random_int_ranged(state).await,
            43 => self.random_float(state).await,
            44 => self.random_double(state).await,
            _ => Err(CpuTrap)
        }
    }
}
