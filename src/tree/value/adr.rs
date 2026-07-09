//! # ADR value codec (RFC 6350 6.3.1, RFC 9554)
//!
//! [`Codec`] for the structured address: seven `;`-separated components,
//! eighteen when any RFC 9554 component carries a value.

use alloc::vec;

use crate::{
    tree::{
        codec::{Codec, encode::encode_component, mode::Escaper},
        value::VcardValueNode,
    },
    value::adr::VcardAdr,
};

impl<'v> Codec<'v> for VcardAdr<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardAdr {
            po_box: node.decode_at(0),
            extended: node.decode_at(1),
            street: node.decode_at(2),
            locality: node.decode_at(3),
            region: node.decode_at(4),
            postal_code: node.decode_at(5),
            country: node.decode_at(6),
            room: node.decode_at(7),
            apartment: node.decode_at(8),
            floor: node.decode_at(9),
            street_number: node.decode_at(10),
            street_name: node.decode_at(11),
            building: node.decode_at(12),
            block: node.decode_at(13),
            subdistrict: node.decode_at(14),
            district: node.decode_at(15),
            landmark: node.decode_at(16),
            direction: node.decode_at(17),
        }
    }

    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
        let mut components = vec![
            encode_component(&self.po_box, escaper),
            encode_component(&self.extended, escaper),
            encode_component(&self.street, escaper),
            encode_component(&self.locality, escaper),
            encode_component(&self.region, escaper),
            encode_component(&self.postal_code, escaper),
            encode_component(&self.country, escaper),
        ];

        if self.has_extended_components() {
            components.extend([
                encode_component(&self.room, escaper),
                encode_component(&self.apartment, escaper),
                encode_component(&self.floor, escaper),
                encode_component(&self.street_number, escaper),
                encode_component(&self.street_name, escaper),
                encode_component(&self.building, escaper),
                encode_component(&self.block, escaper),
                encode_component(&self.subdistrict, escaper),
                encode_component(&self.district, escaper),
                encode_component(&self.landmark, escaper),
                encode_component(&self.direction, escaper),
            ]);
        }

        VcardValueNode::from_components(components, escaper)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec};

    use crate::{
        tree::{
            codec::{Codec, mode::Escaper},
            value::VcardValueNode,
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
            adr.encode(Escaper::Modern).to_string(),
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
            adr.encode(Escaper::Modern).to_string(),
            ";;2875 boul. Laurier;Quebec;;;",
        );
    }
}
