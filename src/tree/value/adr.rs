//! # ADR value codec (RFC 6350 6.3.1, RFC 9554)
//!
//! [`VcardCodec`] for the structured address: seven `;`-separated components,
//! eighteen when any RFC 9554 component carries a value.

use crate::{
    tree::{
        codec::{VcardCodec, encode::VcardValueOut, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::adr::VcardAdr,
};

impl<'v> VcardCodec<'v> for VcardAdr<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        let mut adr = VcardAdr::default();
        adr.decode_into(node);
        adr
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueOut::node(escaper, |out| self.emit(out))
    }
}

impl<'v> VcardAdr<'v> {
    /// Decode `node` into this address, whose components are empty, each
    /// filled where it lies.
    pub(crate) fn decode_into(&mut self, node: &'v VcardValueNode<'_>) {
        node.decode_component_lists_into([
            &mut self.po_box,
            &mut self.extended,
            &mut self.street,
            &mut self.locality,
            &mut self.region,
            &mut self.postal_code,
            &mut self.country,
            &mut self.room,
            &mut self.apartment,
            &mut self.floor,
            &mut self.street_number,
            &mut self.street_name,
            &mut self.building,
            &mut self.block,
            &mut self.subdistrict,
            &mut self.district,
            &mut self.landmark,
            &mut self.direction,
        ]);
    }
}

impl VcardAdr<'_> {
    /// Write the seven classic components, then the eleven RFC 9554 ones when
    /// any of them carries a value.
    pub(crate) fn emit(&self, out: &mut VcardValueOut<'_>) {
        out.component(&self.po_box);
        out.component(&self.extended);
        out.component(&self.street);
        out.component(&self.locality);
        out.component(&self.region);
        out.component(&self.postal_code);
        out.component(&self.country);

        if self.has_extended_components() {
            out.component(&self.room);
            out.component(&self.apartment);
            out.component(&self.floor);
            out.component(&self.street_number);
            out.component(&self.street_name);
            out.component(&self.building);
            out.component(&self.block);
            out.component(&self.subdistrict);
            out.component(&self.district);
            out.component(&self.landmark);
            out.component(&self.direction);
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec};

    use crate::{
        tree::{
            codec::{VcardCodec, mode::VcardEscaper},
            value::node::VcardValueNode,
        },
        value::adr::VcardAdr,
    };

    #[test]
    fn decodes_and_reencodes_the_rfc_9554_components() {
        let node = VcardValueNode::parse(b";;;Quebec;;;Canada;8th wing;;2;2875;boul. Laurier");
        let adr = VcardAdr::decode(&node);

        assert_eq!(adr.locality, vec![Cow::Borrowed("Quebec")]);
        assert_eq!(adr.room, vec![Cow::Borrowed("8th wing")]);
        assert_eq!(adr.floor, vec![Cow::Borrowed("2")]);
        assert_eq!(adr.street_number, vec![Cow::Borrowed("2875")]);
        assert_eq!(adr.street_name, vec![Cow::Borrowed("boul. Laurier")]);
        assert!(adr.has_extended_components());

        // NOTE: eighteen slots come back once any extended component is set.
        assert_eq!(
            adr.encode(VcardEscaper::V4_0).to_string(),
            ";;;Quebec;;;Canada;8th wing;;2;2875;boul. Laurier;;;;;;",
        );
    }

    #[test]
    fn a_classic_address_keeps_its_seven_slots() {
        let adr = VcardAdr {
            street: vec![Cow::Borrowed("2875 boul. Laurier")],
            locality: vec![Cow::Borrowed("Quebec")],
            ..VcardAdr::default()
        };

        assert!(!adr.has_extended_components());
        assert_eq!(
            adr.encode(VcardEscaper::V4_0).to_string(),
            ";;2875 boul. Laurier;Quebec;;;",
        );
    }
}
