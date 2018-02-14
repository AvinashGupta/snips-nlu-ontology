use std::ffi::CString;
use std::ptr::null;
use std::slice;

use libc;
pub use errors::*;

#[repr(C)]
#[derive(Debug)]
pub struct CIntentParserResult {
    pub input: *const libc::c_char,
    pub intent: *const CIntentClassifierResult,
    pub slots: *const CSlotList,
}

impl CIntentParserResult {
    pub fn from(input: ::IntentParserResult) -> OntologyResult<Self> {
        Ok(Self {
            input: CString::new(input.input)?.into_raw(),
            intent: if let Some(intent) = input.intent {
                Box::into_raw(Box::new(CIntentClassifierResult::from(intent)?)) as *const CIntentClassifierResult
            } else {
                null()
            },
            slots: if let Some(slots) = input.slots {
                Box::into_raw(Box::new(CSlotList::from(slots)?)) as *const CSlotList
            } else {
                null()
            },
        })
    }
}

impl Drop for CIntentParserResult {
    fn drop(&mut self) {
        let _ = unsafe { CString::from_raw(self.input as *mut libc::c_char) };
        if !self.intent.is_null() {
            let _ = unsafe { Box::from_raw(self.intent as *mut CIntentClassifierResult) };
        }
        if !self.slots.is_null() {
            let _ = unsafe { Box::from_raw(self.slots as *mut CSlotList) };
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CIntentClassifierResult {
    pub intent_name: *const libc::c_char,
    pub probability: libc::c_float,
}

impl CIntentClassifierResult {
    pub fn from(input: ::IntentClassifierResult) -> OntologyResult<Self> {
        Ok(Self {
            probability: input.probability,
            intent_name: CString::new(input.intent_name)?.into_raw(),
        })
    }
}

impl Drop for CIntentClassifierResult {
    fn drop(&mut self) {
        let _ = unsafe { CString::from_raw(self.intent_name as *mut libc::c_char) };
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CSlotList {
    pub slots: *const CSlot,
    pub size: libc::c_int, // Note: we can't use `libc::size_t` because it's not supported by JNA
}

impl CSlotList {
    pub fn from(input: Vec<::Slot>) -> OntologyResult<Self> {
        Ok(Self {
            size: input.len() as libc::c_int,
            slots: Box::into_raw(
                input
                    .into_iter()
                    .map(|s| CSlot::from(s))
                    .collect::<OntologyResult<Vec<CSlot>>>()?
                    .into_boxed_slice(),
            ) as *const CSlot,
        })
    }
}

impl Drop for CSlotList {
    fn drop(&mut self) {
        let _ = unsafe { Box::from_raw(slice::from_raw_parts_mut(self.slots as *mut CSlot, self.size as usize)) };
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CSlot {
    pub raw_value: *const libc::c_char,
    pub value: CSlotValue,
    pub range_start: libc::c_int,
    pub range_end: libc::c_int,
    pub entity: *const libc::c_char,
    pub slot_name: *const libc::c_char,
}

impl CSlot {
    pub fn from(input: ::Slot) -> OntologyResult<Self> {
        let range = if let Some(range) = input.range {
            range.start as libc::c_int..range.end as libc::c_int
        } else {
            -1..-1
        };

        Ok(Self {
            raw_value: CString::new(input.raw_value)?.into_raw(),
            value: CSlotValue::from(input.value)?,
            range_start: range.start,
            range_end: range.end,
            entity: CString::new(input.entity)?.into_raw(),
            slot_name: CString::new(input.slot_name)?.into_raw(),
        })
    }
}

impl Drop for CSlot {
    fn drop(&mut self) {
        let _ = unsafe { CString::from_raw(self.raw_value as *mut libc::c_char) };
        let _ = unsafe { CString::from_raw(self.entity as *mut libc::c_char) };
        let _ = unsafe { CString::from_raw(self.slot_name as *mut libc::c_char) };
    }
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum CSlotValueType {
    CUSTOM = 1,
    NUMBER = 2,
    ORDINAL = 3,
    INSTANTTIME = 4,
    TIMEINTERVAL = 5,
    AMOUNTOFMONEY = 6,
    TEMPERATURE = 7,
    DURATION = 8,
    PERCENTAGE = 9,
}

impl CSlotValueType {
    pub fn from(slot_value: &::SlotValue) -> Self {
        match slot_value {
            &::SlotValue::Custom(_) => CSlotValueType::CUSTOM,
            &::SlotValue::Number(_) => CSlotValueType::NUMBER,
            &::SlotValue::Ordinal(_) => CSlotValueType::ORDINAL,
            &::SlotValue::InstantTime(_) => CSlotValueType::INSTANTTIME,
            &::SlotValue::TimeInterval(_) => CSlotValueType::TIMEINTERVAL,
            &::SlotValue::AmountOfMoney(_) => CSlotValueType::AMOUNTOFMONEY,
            &::SlotValue::Temperature(_) => CSlotValueType::TEMPERATURE,
            &::SlotValue::Duration(_) => CSlotValueType::DURATION,
            &::SlotValue::Percentage(_) => CSlotValueType::PERCENTAGE,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub enum CPrecision {
    APPROXIMATE = 0,
    EXACT = 1,
}

impl CPrecision {
    pub fn from(value: ::Precision) -> Self {
        match value {
            ::Precision::Approximate => CPrecision::APPROXIMATE,
            ::Precision::Exact => CPrecision::EXACT,
        }
    }
}

pub type CNumberValue = libc::c_double;
pub type CPercentageValue = libc::c_double;
pub type COrdinalValue = libc::c_long;

#[repr(C)]
#[derive(Debug)]
pub enum CGrain {
    YEAR = 0,
    QUARTER = 1,
    MONTH = 2,
    WEEK = 3,
    DAY = 4,
    HOUR = 5,
    MINUTE = 6,
    SECOND = 7,
}

impl CGrain {
    pub fn from(value: ::Grain) -> Self {
        match value {
            ::Grain::Year => CGrain::YEAR,
            ::Grain::Quarter => CGrain::QUARTER,
            ::Grain::Month => CGrain::MONTH,
            ::Grain::Week => CGrain::WEEK,
            ::Grain::Day => CGrain::DAY,
            ::Grain::Hour => CGrain::HOUR,
            ::Grain::Minute => CGrain::MINUTE,
            ::Grain::Second => CGrain::SECOND,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CInstantTimeValue {
    pub value: *const libc::c_char,
    pub grain: CGrain,
    pub precision: CPrecision,
}

impl CInstantTimeValue {
    pub fn from(value: ::InstantTimeValue) -> OntologyResult<Self> {
        Ok(Self {
            value: CString::new(value.value)?.into_raw(),
            grain: CGrain::from(value.grain),
            precision: CPrecision::from(value.precision),
        })
    }
}

impl Drop for CInstantTimeValue {
    fn drop(&mut self) {
        let _ = unsafe { CString::from_raw(self.value as *mut libc::c_char) };
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CTimeIntervalValue {
    pub from: *const libc::c_char,
    pub to: *const libc::c_char,
}

impl CTimeIntervalValue {
    pub fn from(value: ::TimeIntervalValue) -> OntologyResult<Self> {
        Ok(Self {
            from: if let Some(s) = value.from {
                CString::new(s)?.into_raw()
            } else {
                null()
            },
            to: if let Some(s) = value.to {
                CString::new(s)?.into_raw()
            } else {
                null()
            },
        })
    }
}

impl Drop for CTimeIntervalValue {
    fn drop(&mut self) {
        if !self.from.is_null() {
            let _ = unsafe { CString::from_raw(self.from as *mut libc::c_char) };
        }
        if !self.to.is_null() {
            let _ = unsafe { CString::from_raw(self.to as *mut libc::c_char) };
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CAmountOfMoneyValue {
    pub value: libc::c_float,
    pub precision: CPrecision,
    pub unit: *const libc::c_char,
}

impl CAmountOfMoneyValue {
    pub fn from(value: ::AmountOfMoneyValue) -> OntologyResult<Self> {
        Ok(Self {
            value: value.value as libc::c_float,
            precision: CPrecision::from(value.precision),
            unit: if let Some(s) = value.unit {
                CString::new(s)?.into_raw()
            } else {
                null()
            },
        })
    }
}

impl Drop for CAmountOfMoneyValue {
    fn drop(&mut self) {
        if !self.unit.is_null() {
            let _ = unsafe { CString::from_raw(self.unit as *mut libc::c_char) };
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CTemperatureValue {
    pub value: libc::c_float,
    pub unit: *const libc::c_char,
}

impl CTemperatureValue {
    pub fn from(value: ::TemperatureValue) -> OntologyResult<Self> {
        Ok(Self {
            value: value.value as libc::c_float,
            unit: if let Some(s) = value.unit {
                CString::new(s)?.into_raw()
            } else {
                null()
            },
        })
    }
}

impl Drop for CTemperatureValue {
    fn drop(&mut self) {
        if !self.unit.is_null() {
            let _ = unsafe { CString::from_raw(self.unit as *mut libc::c_char) };
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CDurationValue {
    pub years: libc::c_long,
    pub quarters: libc::c_long,
    pub months: libc::c_long,
    pub weeks: libc::c_long,
    pub days: libc::c_long,
    pub hours: libc::c_long,
    pub minutes: libc::c_long,
    pub seconds: libc::c_long,
    pub precision: CPrecision,
}

impl CDurationValue {
    pub fn from(value: ::DurationValue) -> OntologyResult<Self> {
        Ok(Self {
            years: value.years as libc::c_long,
            quarters: value.quarters as libc::c_long,
            months: value.months as libc::c_long,
            weeks: value.weeks as libc::c_long,
            days: value.days as libc::c_long,
            hours: value.hours as libc::c_long,
            minutes: value.minutes as libc::c_long,
            seconds: value.seconds as libc::c_long,
            precision: CPrecision::from(value.precision),
        })
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CSlotValue {
    value_type: CSlotValueType,
    /**
     * Points to either a *const char, a CNumberValue, a COrdinalValue,
     * a CInstantTimeValue, a CTimeIntervalValue, a CAmountOfMoneyValue,
     * a CTemperatureValue or a CDurationValue depending on value_type
     */
    value: *const libc::c_void,
}

impl CSlotValue {
    pub fn from(slot_value: ::SlotValue) -> OntologyResult<Self> {
        let value_type = CSlotValueType::from(&slot_value);
        let value: *const libc::c_void = match slot_value {
            ::SlotValue::Custom(value) => CString::new(value.value.as_bytes())?.into_raw() as _,
            ::SlotValue::Number(value) => Box::into_raw(Box::new(value.value as CNumberValue)) as _,
            ::SlotValue::Ordinal(value) => {
                Box::into_raw(Box::new(value.value as COrdinalValue)) as _
            },
            ::SlotValue::InstantTime(value) => {
                Box::into_raw(Box::new(CInstantTimeValue::from(value)?)) as _
            },
            ::SlotValue::TimeInterval(value) => {
                Box::into_raw(Box::new(CTimeIntervalValue::from(value)?)) as _
            },
            ::SlotValue::AmountOfMoney(value) => {
                Box::into_raw(Box::new(CAmountOfMoneyValue::from(value)?)) as _
            },
            ::SlotValue::Temperature(value) => {
                Box::into_raw(Box::new(CTemperatureValue::from(value)?)) as _
            },
            ::SlotValue::Duration(value) => {
                Box::into_raw(Box::new(CDurationValue::from(value)?)) as _
            },
            ::SlotValue::Percentage(value) => Box::into_raw(Box::new(value.value as CPercentageValue)) as _,
        };
        Ok(Self { value_type, value })
    }
}

impl Drop for CSlotValue {
    fn drop(&mut self) {
        match self.value_type {
            CSlotValueType::CUSTOM => unsafe {
                CString::from_raw(self.value as *mut libc::c_char);
            },
            CSlotValueType::NUMBER => unsafe {
                Box::from_raw(self.value as *mut CNumberValue);
            },
            CSlotValueType::ORDINAL => unsafe {
                Box::from_raw(self.value as *mut COrdinalValue);
            },
            CSlotValueType::INSTANTTIME => unsafe {
                Box::from_raw(self.value as *mut CInstantTimeValue);
            },
            CSlotValueType::TIMEINTERVAL => unsafe {
                Box::from_raw(self.value as *mut CTimeIntervalValue);
            },
            CSlotValueType::AMOUNTOFMONEY => unsafe {
                Box::from_raw(self.value as *mut CAmountOfMoneyValue);
            },
            CSlotValueType::TEMPERATURE => unsafe {
                Box::from_raw(self.value as *mut CTemperatureValue);
            },
            CSlotValueType::DURATION => unsafe {
                Box::from_raw(self.value as *mut CDurationValue);
            },
            CSlotValueType::PERCENTAGE => unsafe {
                Box::from_raw(self.value as *mut CPercentageValue);
            },
        };
    }
}