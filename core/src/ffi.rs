/*
* Copyright 2019 Comcast Cable Communications Management, LLC
*
* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
* http://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License.
*
* SPDX-License-Identifier: Apache-2.0
*/

/*
* Modifications Copyright 2024 Vahab Jabrayilov
* Microsoft Research
* All Rights Reserved.
*/

pub(crate) use dpdk_sys::*;

use crate::warn;
use anyhow::Result;
use dpdk_sys::rte_ether_addr;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::os::raw;
use std::ptr::NonNull;

/// Simplify `*const c_char` or [c_char] to `&str` conversion.
pub(crate) trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for *const raw::c_char {
    #[inline]
    fn as_str(&self) -> &str {
        unsafe {
            CStr::from_ptr(*self).to_str().unwrap_or_else(|_| {
                warn!("invalid UTF8 data");
                Default::default()
            })
        }
    }
}

impl AsStr for [raw::c_char] {
    #[inline]
    fn as_str(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.as_ptr()).to_str().unwrap_or_else(|_| {
                warn!("invalid UTF8 data");
                Default::default()
            })
        }
    }
}

/// Simplify `String` and `&str` to `CString` conversion.
pub(crate) trait ToCString {
    fn into_cstring(self) -> CString;
}

impl ToCString for String {
    #[inline]
    fn into_cstring(self) -> CString {
        CString::new(self).unwrap()
    }
}

impl ToCString for &str {
    #[inline]
    fn into_cstring(self) -> CString {
        CString::new(self).unwrap()
    }
}

/// Simplify dpdk FFI binding's return to a `Result` type.
///
/// # Example
///
/// ```
/// ffi::rte_eth_add_tx_callback(..., ..., ..., ...)
///     .into_result(|_| {
///         DpdkError::new()
/// })?;
/// ```
pub(crate) trait ToResult {
    type Ok;

    fn into_result<E, F>(self, f: F) -> Result<Self::Ok>
    where
        E: Error + Send + Sync + 'static,
        F: FnOnce(Self) -> E,
        Self: Sized;
}

impl<T> ToResult for *mut T {
    type Ok = NonNull<T>;

    #[inline]
    fn into_result<E, F>(self, f: F) -> Result<Self::Ok>
    where
        E: Error + Send + Sync + 'static,
        F: FnOnce(Self) -> E,
    {
        NonNull::new(self).ok_or_else(|| f(self).into())
    }
}

impl<T> ToResult for *const T {
    type Ok = *const T;

    #[inline]
    fn into_result<E, F>(self, f: F) -> Result<Self::Ok>
    where
        E: Error + Send + Sync + 'static,
        F: FnOnce(Self) -> E,
    {
        if self.is_null() {
            Err(f(self).into())
        } else {
            Ok(self)
        }
    }
}

impl ToResult for raw::c_int {
    type Ok = u32;

    #[inline]
    fn into_result<E, F>(self, f: F) -> Result<Self::Ok>
    where
        E: Error + Send + Sync + 'static,
        F: FnOnce(Self) -> E,
    {
        if self >= 0 {
            Ok(self as u32)
        } else {
            Err(f(self).into())
        }
    }
}

pub struct RteEthStats();

impl RteEthStats {
    pub fn default() -> rte_eth_stats {
        rte_eth_stats {
            ipackets: 0,
            opackets: 0,
            ibytes: 0,
            obytes: 0,
            imissed: 0,
            ierrors: 0,
            oerrors: 0,
            rx_nombuf: 0,
            q_ipackets: [0; RTE_ETHDEV_QUEUE_STAT_CNTRS as usize],
            q_opackets: [0; RTE_ETHDEV_QUEUE_STAT_CNTRS as usize],
            q_ibytes: [0; RTE_ETHDEV_QUEUE_STAT_CNTRS as usize],
            q_obytes: [0; RTE_ETHDEV_QUEUE_STAT_CNTRS as usize],
            q_errors: [0; RTE_ETHDEV_QUEUE_STAT_CNTRS as usize],
        }
    }
}

pub struct RteEtherAddr();
impl RteEtherAddr {
    pub fn default() -> rte_ether_addr {
        rte_ether_addr { addr_bytes: [0; 6] }
    }
}

pub struct RteEthDevInfo();
impl RteEthDevInfo {
    pub fn default() -> rte_eth_dev_info {
        rte_eth_dev_info {
            device: std::ptr::null_mut(),
            driver_name: std::ptr::null_mut(),
            if_index: 0,
            min_mtu: 0,
            max_mtu: 0,
            dev_flags: std::ptr::null_mut(),
            min_rx_bufsize: 0,
            max_rx_pktlen: 0,
            max_lro_pkt_size: 0,
            max_rx_queues: 0,
            max_tx_queues: 0,
            max_mac_addrs: 0,
            max_hash_mac_addrs: 0,
            max_vfs: 0,
            max_vmdq_pools: 0,
            rx_seg_capa: RteEthRxsegCapa::default(),
            rx_offload_capa: 0,
            tx_offload_capa: 0,
            rx_queue_offload_capa: 0,
            tx_queue_offload_capa: 0,
            reta_size: 0,
            hash_key_size: 0,
            flow_type_rss_offloads: 0,
            default_rxconf: RteEthRxConf::default(),
            default_txconf: RteEthTxConf::default(),
            vmdq_queue_base: 0,
            vmdq_queue_num: 0,
            vmdq_pool_base: 0,
            rx_desc_lim: RteEthDescLim::default(),
            tx_desc_lim: RteEthDescLim::default(),
            speed_capa: 0,
            nb_rx_queues: 0,
            nb_tx_queues: 0,
            max_rx_mempools: 0,
            default_rxportconf: RteEthDevPortConf::default(),
            default_txportconf: RteEthDevPortConf::default(),
            dev_capa: 0,
            switch_info: RteEthSwitchInfo::default(),
            err_handle_mode: RteEthErrHandleMode::default(),
            reserved_64s: [0; 2],
            reserved_ptrs: [std::ptr::null_mut(); 2],
        }
    }
}

struct RteEthDescLim();
impl RteEthDescLim {
    fn default() -> rte_eth_desc_lim {
        rte_eth_desc_lim {
            nb_max: 0,
            nb_min: 0,
            nb_align: 0,
            nb_seg_max: 0,
            nb_mtu_seg_max: 0,
        }
    }
}

struct RteEthDevPortConf();
impl RteEthDevPortConf {
    fn default() -> rte_eth_dev_portconf {
        rte_eth_dev_portconf {
            burst_size: 0,
            ring_size: 0,
            nb_queues: 0,
        }
    }
}

struct RteEthRxsegCapa();
impl RteEthRxsegCapa {
    fn default() -> rte_eth_rxseg_capa {
        rte_eth_rxseg_capa {
            _bitfield_align_1: [],
            _bitfield_1: Default::default(), // Assuming __BindgenBitfieldUnit has a Default implementation
            max_nseg: 0,
            reserved: 0,
        }
    }
}

struct RteEthRxConf();
impl RteEthRxConf {
    fn default() -> rte_eth_rxconf {
        rte_eth_rxconf {
            rx_thresh: RteEthThresh::default(),
            rx_free_thresh: 0,
            rx_drop_en: 0,
            rx_deferred_start: 0,
            rx_nseg: 0,
            share_group: 0,
            share_qid: 0,
            offloads: 0,
            rx_seg: std::ptr::null_mut(),
            rx_mempools: std::ptr::null_mut(),
            rx_nmempool: 0,
            reserved_64s: [0; 2],
            reserved_ptrs: [std::ptr::null_mut(); 2],
        }
    }
}

struct RteEthTxConf();
impl RteEthTxConf {
    fn default() -> rte_eth_txconf {
        rte_eth_txconf {
            tx_thresh: RteEthThresh::default(),
            tx_rs_thresh: 0,
            tx_free_thresh: 0,
            tx_deferred_start: 0,
            offloads: 0,
            reserved_64s: [0; 2],
            reserved_ptrs: [std::ptr::null_mut(); 2],
        }
    }
}

struct RteEthThresh();
impl RteEthThresh {
    fn default() -> rte_eth_thresh {
        rte_eth_thresh {
            pthresh: 0,
            hthresh: 0,
            wthresh: 0,
        }
    }
}

struct RteEthSwitchInfo();
impl RteEthSwitchInfo {
    fn default() -> rte_eth_switch_info {
        rte_eth_switch_info {
            name: std::ptr::null_mut(),
            domain_id: 0,
            port_id: 0,
            rx_domain: 0,
        }
    }
}

struct RteEthErrHandleMode();
impl RteEthErrHandleMode {
    fn default() -> rte_eth_err_handle_mode {
        0
    }
}
