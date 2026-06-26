//! Building vCard text from the model.

use core::fmt::{self, Display, Formatter};

use crate::rfc6350::{
    vcard::{BEGIN, END, VCARD, Vcard},
    version::{VERSION, VcardVersion},
};

impl Display for Vcard<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{BEGIN}:{VCARD}\r\n{VERSION}:{}\r\n", self.version)?;

        for property in &self.properties {
            write!(
                f,
                "{}{}:{}\r\n",
                property.name, property.params, property.value
            )?;
        }

        write!(f, "{END}:{VCARD}\r\n")
    }
}

impl Display for VcardVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
