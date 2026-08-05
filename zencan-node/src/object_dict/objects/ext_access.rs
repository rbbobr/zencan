use zencan_common::{objects::DataType, objects::{AccessType, ObjectCode}, sdo::AbortCode};
use crate::object_dict::ObjectAccess;

/// Трейт-расширение для удобного чтения и записи типизированных значений.
/// Эти методы НЕ попадают в V-Table `dyn ObjectAccess`, поэтому не раздувают бинарник!
pub trait ObjectAccessExt {
    fn read_u32(&self, sub: u8) -> Result<u32, AbortCode>;
    fn read_u16(&self, sub: u8) -> Result<u16, AbortCode>;
    fn read_u8(&self, sub: u8) -> Result<u8, AbortCode>;
    fn read_i32(&self, sub: u8) -> Result<i32, AbortCode>;
    fn read_i16(&self, sub: u8) -> Result<i16, AbortCode>;
    fn read_i8(&self, sub: u8) -> Result<i8, AbortCode>;

    /// Get the highest sub index available in this object
    fn max_sub_number(&self) -> u8;
    fn access_type(&self, sub: u8) -> Result<AccessType, AbortCode>;
    fn data_type(&self, sub: u8) -> Result<DataType, AbortCode>;
    fn size(&self, sub: u8) -> Result<usize, AbortCode>;
    fn current_size(&self, sub: u8) -> Result<usize, AbortCode>;


}

/// Бланкетная реализация для ЛЮБОГО типа, реализующего ObjectAccess.
/// Связка `?Sized` критически важна, чтобы это работало для `&dyn ObjectAccess`.
impl<T: ObjectAccess + ?Sized> ObjectAccessExt for T {

    /// Get the highest sub index available in this object
    fn max_sub_number(&self) -> u8 {
        match self.object_code() {
            ObjectCode::Array | ObjectCode::Record => {
                let mut buf = [0u8];
                // Читаем 1 байт из подабъекта 0 (где хранится max sub number)
                if self.read(0, 0, &mut buf).is_ok() {
                    buf[0]
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    /// Get the access type of a specific sub object
    fn access_type(&self, sub: u8) -> Result<AccessType, AbortCode> {
        Ok(self.sub_info(sub)?.access_type)
    }

    /// Get the data type of a specific sub object
    fn data_type(&self, sub: u8) -> Result<DataType, AbortCode> {
        Ok(self.sub_info(sub)?.data_type)
    }

    /// Get the maximum size of an sub object
    ///
    /// For most sub objects, this matches the current_size, but for strings the size of the
    /// currently stored value (returned by `current_size()`) may be smaller.
    fn size(&self, sub: u8) -> Result<usize, AbortCode> {
        Ok(self.sub_info(sub)?.size)
    }

    /// Get the current size of a sub object
    ///
    /// Note that this is not necessarily the allocated size of the object, as some objects (such as
    /// strings) may have values shorter than their maximum size. As such, this gives the maximum
    /// number of bytes which may be read, but not necessarily the number of bytes which may be
    /// written.
    fn current_size(&self, sub: u8) -> Result<usize, AbortCode> {
        const CHUNK_SIZE: usize = 8;

        let size = self.size(sub)?;
        if self.data_type(sub)?.is_str() {
            // Look for first 0
            let mut chunk = 0;
            let mut buf = [0; CHUNK_SIZE];
            while chunk < size / CHUNK_SIZE + 1 {
                let offset = chunk * CHUNK_SIZE;
                let bytes_to_read = (size - offset).min(CHUNK_SIZE);
                self.read(sub, offset, &mut buf[0..bytes_to_read])?;

                if let Some(zero_pos) = buf[0..bytes_to_read].iter().position(|b| *b == 0) {
                    return Ok(zero_pos + chunk * CHUNK_SIZE);
                }
                chunk += 1;
            }
        }
        // not a string type or no null-terminator was found
        Ok(size)
    }

    

    /// Read a sub object as a u32
    fn read_u32(&self, sub: u8) -> Result<u32, AbortCode> {
        let mut buf = [0; 4];
        self.read(sub, 0, &mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    /// Read a sub object as a u16
    fn read_u16(&self, sub: u8) -> Result<u16, AbortCode> {
        let mut buf = [0; 2];
        self.read(sub, 0, &mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    /// Read a sub object as a u8
    fn read_u8(&self, sub: u8) -> Result<u8, AbortCode> {
        let mut buf = [0; 1];
        self.read(sub, 0, &mut buf)?;
        Ok(buf[0])
    }

    /// Read a sub object as an i32
    fn read_i32(&self, sub: u8) -> Result<i32, AbortCode> {
        let mut buf = [0; 4];
        self.read(sub, 0, &mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    /// Read a sub object as an i16
    fn read_i16(&self, sub: u8) -> Result<i16, AbortCode> {
        let mut buf = [0; 2];
        self.read(sub, 0, &mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    /// Read a sub object as an i8
    fn read_i8(&self, sub: u8) -> Result<i8, AbortCode> {
        let mut buf = [0; 1];
        self.read(sub, 0, &mut buf)?;
        Ok(buf[0] as i8)
    }
    
    
}