# Wio-LR2021 Module

**LoRa® WirelessModule LR2021**

**Datasheet**

**v1.1**

## Content

1. Introduction … 1  
1.1. Feature … 1  
1.2. Diagram … 3  
2. Description … 4  
2.1. Pin Definition … 4  
2.2. DIO Specification … 5  
3. Electrical Characteristics … 6  
3.1. Absolute Maximum Ratings … 6  
3.2. Normal Working Conditions … 6  
3.3. Digital I/O Characteristics … 7  
4. RF Performance … 8  
4.1. General RF Parameters … 8  
4.2. Power Consumption … 9  
5. Application Information … 14  
5.1. Mechanism Dimension … 14  
5.2. Reference Design … 15  
5.3. Host Interface Timing … 16  
5.3.1. SPI Timing Specification … 16  
5.3.2. BUSY Signal Specification … 17  
6. Production … 18  
6.1 Reflow Soldering Parameters … 18  
6.2 Tape & Reel … 19  
7. Version … 21

## 1. Introduction

Wio-LR2021 is a next-generation, multi-band wireless transceiver module powered by Semtech’s fourth-generation LR2021 chipset. Engineered as a unified RF platform, it integrates Sub-GHz (863–928 MHz), 2.4 GHz ISM, and licensed S-band (1.6–2.4 GHz) operation into a single compact subsystem, eliminating the need for multiple radio designs across regional and application boundaries.

![Picture on page 3](images/p003-picture-01.png)

> **Image description:** Close-up photo of the top surface of an electronic module.
>
> Visible markings include:
> - seed studio
> - LR2021 v1.0
>
> No dimensions or scale are shown.

![Picture on page 3](images/p003-picture-02.png)

> **Image description:** Photo of the top side of an electronic module.
>
> Visible markings include:
> seeded studio
> Wio-LR2021
> FCC ID: Z4T-WIO-LR2021
> . FCC
> CE
>
> No dimensions or scale are shown.

### 1.1. Feature

### Unified Multi-Band RF Platform

- Sub-GHz: 863~928 MHz (868/915 MHz ISM bands)<sup>1</sup>, receive sensitivity down to –141.1 dBm (SF12/125 kHz), transmit power up to +22 dBm
- 2.4 GHz ISM: 2400–2500 MHz global operation, sensitivity down to –133 dBm, supporting data rates up to 2.6 Mbps (FLRC modulation)
- S-Band: 1.6–2.4 GHz licensed spectrum for satellite IoT (NTN), sensitivity down to –131 dBm

### Ultra-Low Power Consumption

- Transmit: 116 mA (Sub-GHz @ +22 dBm), 28.4 mA (2.4 GHz @ +12 dBm)
- Receive: 8.19 mA (Sub-GHz), 8.59 mA (2.4 GHz)
- Deep sleep: 583 nA
- Idle: 960 uA

### Advanced Modulation & Protocol Support

- Modulation: LoRa, (G)FSK, (G)MSK, FLRC, 4-FSK, O-QPSK, and LR-FHSS (transmit-only) for enhanced anti-interference
- Protocol stack<sup>2</sup> : LoRaWAN (Sub-GHz and 2.4 GHz), Bluetooth® LE 5.0, IEEE® 802.15.4 (Thread®/Zigbee™), Wi-SUN, Wireless M-BUS
- Concurrent multi-Spreading Factor (SF) reception and enhanced Channel Activity Detection (CAD) for high-density deployments

**NOTE:**

1\. While the Semtech LR2021 transceiver covers the 150–960 MHz range, the Wio-LR2021 module’s RF front-end including the matching network and harmonic filtering is factory-optimized for the 868 MHz and 915 MHz ISM bands to ensure certified RF performance and regulatory compliance.

For deployments requiring the 433–490 MHz band, a dedicated low-frequency variant, designated Wio-LR2021-LF, is currently under development for native operation in the low-frequency band. Contact sensecap@seeed.cc for feasibility and lead-time assessment.

2\. Protocols such as BLE, Thread, Zigbee, Wi‑SUN, and Z‑Wave require the main MCU to run the corresponding MAC/network protocol stack, whereas the Wio‑LR2021 only provides support for physical‑layer modulation/demodulation and packet processing.

### 1.2. Diagram

![Figure 1 Wio-LR2021 Schematic Diagram](images/p005-diagram-01.png)

> **Image description:** Block diagram of the Wio-LR2021 Schematic Diagram.
>
> Blocks and Components:
> LR2021
> VCC
> VCC_IN
> RFI_HF
> 2.4GHz RF
> SPI
> SetRegMode()
> RFO_HF
> NRST
> MCU
> DIO
> SIMO / LDO
> VR_PA
> RFO_LF1
> Sub_GHz RF
> RFO_LF2
> RFI_LF
> XTAL
> VTCXO
> 32 MHz
> TCXO
>
> Connections:
> MCU connects to VCC (output arrow), VCC_IN (output arrow), SPI (bidirectional arrow), NRST (bidirectional arrow), and DIO (bidirectional arrow).
> LR2021 connects to VCC_IN (input arrow).
> LR2021 connects to SPI (input arrow).
> LR2021 connects to NRST (input arrow).
> LR2021 connects to DIO (input arrow).
> LR2021 connects to SetRegMode() (input arrow).
> LR2021 connects to SIMO / LDO (input arrow).
> LR2021 connects to RFI_HF (output arrow).
> LR2021 connects to RFO_HF (output arrow).
> LR2021 connects to VR_PA (output arrow).
> LR2021 connects to RFO_LF1 (output arrow).
> LR2021 connects to RFO_LF2 (output arrow).
> LR2021 connects to RFI_LF (output arrow).
> LR2021 connects to XTAL (input arrow).
> LR2021 connects to VTCXO (input arrow).
> RFI_HF connects to 2.4GHz RF (bidirectional arrow).
> RFO_HF connects to 2.4GHz RF (bidirectional arrow).
> VR_PA connects to 2.4GHz RF (bidirectional arrow).
> RFO_LF1 connects to Sub_GHz RF (bidirectional arrow).
> RFO_LF2 connects to Sub_GHz RF (bidirectional arrow).
> RFI_LF connects to Sub_GHz RF (bidirectional arrow).
> XTAL connects to 32 MHz TCXO (bidirectional arrow).
> VTCXO connects to 32 MHz TCXO (bidirectional arrow).
>
> Numbers and Values:
> 32 MHz is associated with TCXO.

**Figure 1 Wio-LR2021 Schematic Diagram**

![Figure 2 LR2021 Block Diagram](images/p005-diagram-02.png)

> **Image description:** Block diagram of LR2021.
>
> Blocks:
> RFI_HF
> LNA HF
> Mix HF
> AAF
> ADC
> Digital Front End
> LoRa Demod
> RFI FIFO
> LNA LF
> Mix LF
> PA control
> Generic modulator
> CPFSK, LR-FHSS, BPSK, FLRC
> Packet controller
> RX FIFO
> SPI
> BUSY
> NSS
> SCK
> MOSI
> MISO
> GND_LF
> RFI_LF
> PA HF
> PLL
> PLL control
> RFO_HF
> RFO_LF1
> PA LF
> LoRa modulator
> RFO_LF2
> VDCC2
> VR PA
> REG_PA
> HFXOSC 32 MHz
> LFRCOSC 32 KHz
> HFRCOSC 32 MHz
> REG TCXO
> SIMO
> LR2021
> DIO5
> DIO6
> DIO7
> DIO8
> DIO9
> DIO10
> DIO11
> NRESET
> VPAX2
> XTA
> XTB
> NTC
> VTCX0
> VDCC1
> VPAX1
> LXB
> GND DCC
> LXA
> VBAT
>
> Connections:
> RFI_HF connects to LNA HF.
> GND_LF connects to Mix HF.
> RFI_LF connects to LNA LF.
> Mix HF connects to AAF.
> AAF connects to ADC.
> ADC connects to Digital Front End.
> Digital Front End connects to LoRa Demod.
> LoRa Demod connects to RFI FIFO.
> LNA LF connects to Mix LF.
> Mix LF connects to PA control.
> PA control connects to PLL.
> PLL connects to PLL control.
> PLL control connects to Generic modulator.
> Generic modulator connects to LoRa modulator.
> LoRa modulator connects to Packet controller.
> Packet controller connects to RX FIFO.
> RFI FIFO connects to SPI.
> SPI connects to BUSY.
> SPI connects to NSS.
> SPI connects to SCK.
> SPI connects to MOSI.
> SPI connects to MISO.
> PA HF connects to RFO_HF.
> PA LF connects to RFO_LF1.
> RFO_HF connects to PLL.
> RFO_LF1 connects to PLL.
> RFO_LF2 connects to PLL.
> VDCC2 connects to LR2021.
> VR PA connects to LR2021.
> REG_PA connects to LR2021.
> HFXOSC 32 MHz connects to LR2021.
> LFRCOSC 32 KHz connects to LR2021.
> HFRCOSC 32 MHz connects to LR2021.
> REG TCXO connects to LR2021.
> SIMO connects to LR2021.
> LR2021 connects to DIO5.
> LR2021 connects to DIO6.
> LR2021 connects to DIO7.
> LR2021 connects to DIO8.
> LR2021 connects to DIO9.
> LR2021 connects to DIO10.
> LR2021 connects to DIO11.
> LR2021 connects to NRESET.
> VPAX2 connects to LR2021.
> XTA connects to LR2021.
> XTB connects to LR2021.
> NTC connects to LR2021.
> VTCX0 connects to LR2021.
> VDCC1 connects to LR2021.
> VPAX1 connects to LR2021.
> LXB connects to LR2021.
> GND DCC connects to LR2021.
> LXA connects to LR2021.
> VBAT connects to LR2021.
>
> Numbers and Units:
> HFXOSC 32 MHz
> LFRCOSC 32 KHz
> HFRCOSC 32 MHz

**Figure 2 LR2021 Block Diagram**

## 2. Description

### 2.1. Pin Definition

![Picture on page 6](images/p006-picture-01.png)

> **Image description:** Rendering of an electronic component package, viewed from the top.
>
> The component has a rectangular body with a green border.
> There is a small circular marking near the bottom center of the package.
> No labels, part numbers, logos, connectors, or pin markings are visible.
> No dimensions or scale are shown.

| Number | Name | Type | Description |
| --- | --- | --- | --- |
| 1 | VCC_IN | Power | Power supply |
| 2 | DIO5 | I/O | Multi-purpose digital I/O |
| 3 | RESET | I | Reset signal,active LOW |
| 4 | SPI_MISO | O | SPI MISO |
| 5 | SPI_MOSI | I | SPI MOSI |
| 6 | SPI_SCK | I | SPI clock |
| 7 | SPI_NSS | I | SPI chip select |
| 8 | BUSY | O | Busy indicator signal |
| 9 | GND | - | Ground |
| 10 | SubG_RF | RF | Sub-GHz channel |
| 11 | GND | - | Ground |
| 12 | GND | - | Ground |
| 13 | 2.4G_RF | RF | 2.4GHz channel |
| 14 | GND | - | Ground |
| 15 | GND | - | Ground |
| 16 | DIO11 | I/O | Multi-purpose digital I/O |
| 17 | DIO10 | I/O | Multi-purpose digital I/O |
| 18 | DIO9 | I/O | Multi-purpose digital I/O |
| 19 | DIO8 | I/O | Multi-purpose digital I/O |
| 20 | GND | - | Ground |
| 21 | DIO6 | I/O | Multi-purpose digital I/O |
| 22 | DIO7 | I/O | Multi-purpose digital I/O |

Note:  
1. The LR2021 only features 7 multi-purpose I/O channels (DIO5~DIO11); it does not have DIO0~DIO4 pins. Wio-LR2021 module has exposed all 7 DIO channels.

**Figure 3 Wio-LR2021 Pinout**

### 2.2. DIO Specification

All DIO functions (IRQ, RF switch control, clock output, Tx/Rx trigger) must be configured viaSPI command *SetDioFunction* while the module is in STDBY_RC mode. Configuration is retained across Sleep (warm start) but lost after a cold start or power cycle.

Upon module power-up or reset release, the LR2021 transceiver enters STDBY_RC mode with the following default pin states:

- DIO5, DIO6: Internal weak pull-up to VCC_IN. These two pins will read logic-high if left unconnected.
- DIO7~DIO11: High-impedance (Hi-Z). External pull-up or pull-down resistors are recommended if these pins are routed to the host PCB and not immediately configured by firmware.

## 3. Electrical Characteristics

### 3.1. Absolute Maximum Ratings

Please note that exceeding these values will result in permanent damage and functional degradation.

| Item | Value |
| --- | --- |
| Supply Voltage (Vcc) | 3.9 V |
| IO Voltage | 3.9 V |
| Operating Temperature | -40 ~ 85 °C |
| Storage Temperature | 23 ± 5 °C |
| Maximum RF Input Power | +10 dBm<sup>[1]</sup> |

**CAUTION:**  
This module employs a switchless Direct-Tie matching network where the PA output (RFO) and LNA input (RFI) share the same RF node. Consequently, any external power applied to the antenna port is presented simultaneously to both the RFI and RFO pins of the LR2021.  
[1] The +10 dBm absolute maximum is a stress rating only. Functional operation at or above **+6 dBm** is not recommended and may degrade device reliability. External power above +6 dBm requires additional front-end protection.

### 3.2. Normal Working Conditions

<table>
<thead>
<tr><th></th><th>Min</th><th>Typical</th><th>Max</th><th>Units</th><th>Remark</th></tr>
</thead>
<tbody>
<tr><td>Supply Voltage (Vcc)</td><td>1.8</td><td>3.3</td><td>3.7</td><td>V</td><td>Ripple: 11.2mV, BW: 20MHz</td></tr>
<tr><td>VTCXO (Internal)</td><td>3.135</td><td>3.3</td><td>3.465</td><td>V</td><td>Ripple: 8.129 mV, BW: 20MHz</td></tr>
<tr><td rowspan="4">Transmit Current</td><td>-</td><td>-</td><td>-</td><td>mA</td><td>490MHz @ 17dBm</td></tr>
<tr><td>-</td><td>38.5</td><td>-</td><td>mA</td><td>868MHz @ 14dBm</td></tr>
<tr><td>-</td><td>119.4</td><td>-</td><td>mA</td><td>915MHz @ 22dBm</td></tr>
<tr><td>-</td><td>28.4</td><td>-</td><td>mA</td><td>2.4GHz @ 12dBm</td></tr>
<tr><td rowspan="2">Receive Current</td><td>-</td><td>8.19</td><td>-</td><td>mA</td><td>Sub-GHz LoRa @ SF12, 125kHz</td></tr>
<tr><td>-</td><td>8.59</td><td>-</td><td>mA</td><td>2.4GHz LoRa @ SF12, 125kHz</td></tr>
<tr><td rowspan="4"></td><td>-</td><td>7.99</td><td>-</td><td>mA</td><td>915MHz FLRC @ 260kbps</td></tr>
<tr><td>-</td><td>11.92</td><td>-</td><td>mA</td><td>915MHz FLRC @ 2600kbps</td></tr>
<tr><td>-</td><td>8.52</td><td>-</td><td>mA</td><td>2.4GHz FLRC @ 260kbps</td></tr>
<tr><td>-</td><td>12.60</td><td>-</td><td>mA</td><td>2.4GHz FLRC @ 2600kbps</td></tr>
<tr><td>Sleep Current</td><td>-</td><td>583</td><td>-</td><td>nA</td><td></td></tr>
<tr><td>Idle Current</td><td>-</td><td>960</td><td>-</td><td>uA</td><td></td></tr>
</tbody>
</table>

### 3.3. Digital I/O Characteristics

|   | Min | Typical | Max | Units | Remark |
| --- | --- | --- | --- | --- | --- |
| Output Logic 1 (VOH) | 0.9×VCC | - | VCC | V | I<sub>max</sub>=2.5mA |
| Output Logic 0 (VOL) | 0 | - | 0.1×VCC | V | I<sub>max</sub>=-2.5mA |
| Input Logic 1 (VIH) | 0.7×VCC | - | VCC+0.3 | V |   |
| Input Logic 0 (VIL) | -0.3 | - | 0.3×VCC | V |   |
| Internal pull-up/pull-down resistors | - | 40 | - | kΩ |   |

## 4. RF Performance

### 4.1. General RF Parameters

| Parameter | Sub-GHz | 2.4 GHz | Remark |
| --- | --- | --- | --- |
| Frequency Range | 863 ~ 928 MHz | 2402 ~ 2480 MHz |   |
| Tx Power Range | -10 ~ +22 dBm | -17 ~ +12 dBm | Software configurable |
| Frequency Drift | &lt; 1.5 ppm | &lt; 1.5 ppm |   |

<table>
<thead>
<tr><th>Parameter</th><th>Value</th><th>Remark</th></tr>
</thead>
<tbody>
<tr><td rowspan="4">LoRa Receiving Sensitivity<sup>[1]</sup></td><td>-127.5 dBm</td><td>Sub-GHz @ SF7, 125kHz</td></tr>
<tr><td>-141.1 dBm</td><td>Sub-GHz @ SF12, 125kHz</td></tr>
<tr><td>-118.4 dBm</td><td>2.4GHz @ SF7, 400kHz</td></tr>
<tr><td>-133 dBm</td><td>2.4GHz @ SF12, 400kHz</td></tr>
<tr><td rowspan="4">FLRC Receiving Sensitivity<sup>[1]</sup></td><td>-110 dBm</td><td>915MHz @ BRF = 260kbps, CR = 3/4, BWF = 300kHz</td></tr>
<tr><td>-100.5 dBm</td><td>915MHz @ BRF = 2600 kbps, CR = 3/4, BWF = 2666 kHz</td></tr>
<tr><td>-108.5 dBm</td><td>2.4GHz @ BRF = 260kbps, CR = 3/4, BWF = 300kHz</td></tr>
<tr><td>-99 dBm</td><td>2.4GHz @ BRF = 2600kbps, CR = 3/4, BWF = 2666kHz</td></tr>
<tr><td rowspan="3">Harmonic Noise</td><td rowspan="3">-</td><td>490MHz @ 120kHz</td></tr>
<tr><td>868MHz @ 120kHz</td></tr>
<tr><td>915MHz @ 100kHz</td></tr>
</tbody>
</table>

**NOTE:**

[1] The sensitivity values are referenced to the signal generator output; cable loss is not compensated in the raw data. The approximate cable loss is 0.7 dB. Condition: PER ≤ 10 %, RxBoosted ON.

### 4.2. Power Consumption

#### 4.2.1. LoRa Power Specifications

### Sub-GHz Transmission Power Consumption

<table>
<thead>
<tr><th colspan="2">868 MHz</th><th colspan="2">915 MHz</th><th colspan="2">490 MHz</th></tr>
<tr><th>Tx Power (dBm)</th><th>Current (mA)</th><th>Tx Power (dBm)</th><th>Current (mA)</th><th>Tx Power (dBm)</th><th>Current (mA)</th></tr>
</thead>
<tbody>
<tr><td>-10</td><td>7.8</td><td>-10</td><td>7.8</td><td>-</td><td>-</td></tr>
<tr><td>0</td><td>9.9</td><td>0</td><td>10.1</td><td>-</td><td>-</td></tr>
<tr><td>+10</td><td>19.4</td><td>+10</td><td>19.7</td><td>-</td><td>-</td></tr>
<tr><td>+15</td><td>38.5</td><td>+15</td><td>38.5</td><td>-</td><td>-</td></tr>
<tr><td>+17</td><td>52.3</td><td>+17</td><td>53.9</td><td>-</td><td>-</td></tr>
<tr><td>+18</td><td>61.8</td><td>+18</td><td>63.4</td><td>-</td><td>-</td></tr>
<tr><td>+19</td><td>76.0</td><td>+19</td><td>77.8</td><td>-</td><td>-</td></tr>
<tr><td>+20</td><td>92.5</td><td>+20</td><td>93.4</td><td>-</td><td>-</td></tr>
<tr><td>+21</td><td>109.8</td><td>+21</td><td>111.2</td><td>-</td><td>-</td></tr>
<tr><td>+22</td><td>116.0</td><td>+22</td><td>119.4</td><td>-</td><td>-</td></tr>
</tbody>
</table>

NOTE: Tested at 25 °C ambient.

![Figure 4 Output Power VS Transmission Current (868MHz)](images/p011-figure-01.png)

> **Image description:** Graph of Output Power VS Transmission Current (868MHz).
> The x-axis plots Tx Current (mA) with a range from 0 to 120.
> The y-axis plots Output Power (dBm) with a range from -15 to 25.
> There is one curve representing the Sub-GHz Band (868 MHz). The overall trend of the curve is an increasing relationship between Tx Current and Output Power.
> The values printed in the figure are:
> 10 dBm, 19.4 mA
> 15 dBm, 38.5 mA
> 22 dBm, 116.0 mA

**Figure 4 Output Power VS Transmission Current (868MHz)**

![Figure on page 12](images/p012-figure-01.png)

> **Image description:** Graph of Output Power VS Transmission Current (915MHz).
>
> The vertical axis plots Output Power in dBm, ranging from -15 to 25. The horizontal axis plots Tx Current in mA, ranging from 0 to 120.
>
> There is one curve shown, which represents the relationship between Output Power and Tx Current for the Sub-GHz Band (915 MHz). The overall trend of the curve is that the Output Power increases as the Tx Current increases.
>
> Values printed in the figure as text include:
> - 10 dBm, 19.7 mA
> - 15 dBm, 38.5 mA
> - 22 dBm, 119.4 mA
> - Figure 5 Output Power VS Transmission Current (915MHz)

### 2.4GHz Transmission Power Consumption

| Tx Power (dBm) | Current (mA) |
| --- | --- |
| -17 | 9.7 |
| -13 | 10.2 |
| -10 | 10.8 |
| -5 | 12.0 |
| -3 | 12.6 |
| 0 | 13.6 |
| +3 | 15.3 |
| +5 | 17.0 |
| +10 | 22.9 |
| +12 | 28.4 |

NOTE: Tested at 25 °C ambient.

![Figure on page 13](images/p013-figure-01.png)

> **Image description:** Graph of Output Power VS Transmission Current (2.4GHz).
>
> The vertical axis plots Output Power in dBm, ranging from -20 to 15. The horizontal axis plots Tx Current in mA, ranging from 0 to 35.
>
> There is one curve shown, which represents the relationship between Output Power and Tx Current. The overall trend of the curve is that the Output Power increases as the Tx Current increases, reaching a peak and then slightly decreasing at the highest measured current.
>
> Values printed in the figure as text include:
> - 0 dBm, 13.6 mA
> - -17 dBm, 9.7 mA
> - 12 dBm, 28.4 mA
>
> Figure 6 Output Power VS Transmission Current (2.4GHz)

#### 4.2.2. FLRC Power Specifications

### 260kbps Bit Rate Transmission Power Consumption

<table>
<thead>
<tr><th colspan="2">915MHz</th><th colspan="2">2.4GHz</th></tr>
<tr><th>Tx Power (dBm)</th><th>Current (mA)</th><th>Tx Power (dBm)</th><th>Current (mA)</th></tr>
</thead>
<tbody>
<tr><td>-10</td><td>8.93</td><td>-17</td><td>10.13</td></tr>
<tr><td>-5</td><td>9.69</td><td>-14</td><td>11.07</td></tr>
<tr><td>-3</td><td>9.82</td><td>-11</td><td>11.53</td></tr>
<tr><td>+2</td><td>10.87</td><td>-7</td><td>11.54</td></tr>
<tr><td>+4</td><td>12.30</td><td>-4</td><td>12.77</td></tr>
<tr><td>+8</td><td>17.04</td><td>-2</td><td>13.57</td></tr>
<tr><td>+12</td><td>25.26</td><td>0</td><td>14.38</td></tr>
<tr><td>+16</td><td>46.82</td><td>+5</td><td>17.30</td></tr>
<tr><td>+19</td><td>75.91</td><td>+8</td><td>19.51</td></tr>
<tr><td>+22</td><td>117.53</td><td>+12</td><td>27.58</td></tr>
</tbody>
</table>

![Figure 7 Output Power VS Transmission Current (FLRC @ 260kbps)](images/p014-figure-01.png)

> **Image description:** Graph of Output Power VS Transmission Current (FLRC @ 260kbps).
>
> The x-axis plots Output Power in dBm, ranging from -20 to 20.
> The y-axis plots Tx Current in mA, ranging from 0 to 120.
>
> There are two curves:
> - 915MHz: This curve shows a generally increasing trend, starting at a low current and rising sharply as the output power increases.
> - 2.4GHz: This curve also shows an increasing trend, but it remains consistently below the 915MHz curve across the entire range of output power.
>
> Text printed in the figure includes:
> FLRC @ 260kbps
> Output Power (dBm)
> Tx Current (mA)
> 915MHz
> 2.4GHz

**Figure 7 Output Power VS Transmission Current (FLRC @ 260kbps)**

### 2600kbps Bit Rate Transmission Power Consumption

<table>
<thead>
<tr><th colspan="2">915MHz</th><th colspan="2">2.4GHz</th></tr>
<tr><th>Tx Power (dBm)</th><th>Current (mA)</th><th>Tx Power (dBm)</th><th>Current (mA)</th></tr>
</thead>
<tbody>
<tr><td>-10</td><td>2.60</td><td>-17</td><td>2.77</td></tr>
<tr><td>-5</td><td>2.72</td><td>-14</td><td>2.81</td></tr>
<tr><td>-3</td><td>2.73</td><td>-11</td><td>2.84</td></tr>
<tr><td>+2</td><td>2.91</td><td>-9</td><td>2.93</td></tr>
<tr><td>+4</td><td>3.05</td><td>-5</td><td>3.02</td></tr>
<tr><td>+7</td><td>3.26</td><td>-2</td><td>3.13</td></tr>
<tr><td>+10</td><td>3.76</td><td>0</td><td>3.17</td></tr>
<tr><td>+15</td><td>5.50</td><td>+5</td><td>3.45</td></tr>
<tr><td>+18</td><td>7.62</td><td>+8</td><td>3.64</td></tr>
<tr><td>+22</td><td>12.07</td><td>+12</td><td>4.40</td></tr>
</tbody>
</table>

![Figure on page 15](images/p015-figure-01.png)

> **Image description:** Graph of Output Power VS Transmission Current (FLRC @ 2600kbps).
>
> The vertical axis plots Tx Current in mA, ranging from 2 to 14. The horizontal axis plots Output Power in dBm, ranging from -20 to 20.
>
> There are two curves:
> - 915MHz: This curve shows a generally increasing trend, starting around 2.5 mA at -20 dBm and rising to approximately 12 mA at 20 dBm.
> - 2.4GHz: This curve also shows an increasing trend, starting around 2.5 mA at -20 dBm and rising to approximately 4.5 mA at 20 dBm.
>
> Text printed in the figure includes:
> - Title: FLRC @ 2600kbps
> - Legend entries: 915MHz and 2.4GHz
> - Figure caption: Figure 8 Output Power VS Transmission Current (FLRC @ 2600kbps)

## 5. Application Information

### 5.1. Mechanism Dimension (Unit: mm)

![Figure 9 Mechanical Drawing](images/p016-diagram-01.png)

> **Image description:** Package drawing of the top view of the module. Pin 1 is not explicitly indicated.
>
> Pin numbering is not provided in the drawing.
>
> Dimensions:
> 17.0688
> 1.0000
> 0.6000
> 2.2000
> 0.2768
> 1.2700
> 3.0920
> 10.4140
> 9.4066
> 8.6000
> 2.8000
> 16.7000

**Figure 9 Mechanical Drawing**

![Figure 10 Footprint Drawing](images/p016-figure-02.png)

> **Image description:** Pinout or package drawing. Top view. Pin 1 is not explicitly marked.
>
> Pins are not explicitly numbered or named in the drawing.
>
> Dimensions:
> 0.26
> 1.27
> 3.08
> 0.52
> 10.42
> 13.75
> 1.50
> 1.00
> 17.07

**Figure 10 Footprint Drawing**

### 5.2. Reference Design

The following is a typical reference design using the Wio-LR2021 module:

![Figure 11 Reference Design Based on Wio-LR2021](images/p017-diagram-01.png)

> **Image description:** Schematic of Figure 11 Reference Design Based on Wio-LR2021.
>
> Blocks and Components:
> U1 (MCU)
> U2
> U3
> GND
> C4
> NC
> 0R
> C5
> R10
> C3
> WIO-LR2021
> R11
> C6
> 12
> 13
> 2.4G_RF
> SUBG_RF
> 10
> 9
> BUSY
> 8
> R4
> 470R
> 14
> 15
> R5
> NSS
> DIO11
> R1
> 16
> 7
> R6
> SCK
> 6
> RI010
> 17
> DIO10
> MCU
> RI09
> 18
> DIO9
> MOSI
> 5
> R7
> DIO8
> R2
> 19
> MISO
> 4
> R8
> 20
> RESET
> 3
> R9
> NRST
> VCC_IN
> RI07
> DIO7
> RI3
> 22
> DIO6
> RI05
> 2
> 1
> FB1
> C2
> 100nF
> C1
> 10uF
>
> Connections:
> U1 connects to:
> BUSY (to 9)
> NSS (to 7)
> SCK (to 6)
> MOSI (to 5)
> MISO (to 19)
> NRST (to 3)
> DIO11 (to R1)
> DIO8 (to R2)
> DIO7 (to R3)
>
> U2 connects to:
> GND (to GND)
> C4 (to GND)
> NC (to GND)
> 0R (to GND)
>
> U3 connects to:
> GND (to GND)
> C5 (to GND)
> NC (to GND)
> 0R (to GND)
>
> WIO-LR2021 connects to:
> 12 (to GND)
> 13 (to GND)
> 2.4G_RF (to GND)
> 14 (to GND)
> 15 (to GND)
> 16 (to DIO11)
> 17 (to DIO10)
> 18 (to DIO9)
> 19 (to DIO8)
> 20 (to GND)
> 21 (to GND)
> 22 (to DIO6)
>
> MCU connects to:
> DIO11 (to R1)
> DIO8 (to R2)
> DIO7 (to R3)
>
> Connections involving resistors and capacitors:
> R1 connects DIO11 to 16.
> R2 connects DIO8 to 17.
> R3 connects DIO7 to 18.
> R4 connects 8 to 470R.
> R5 connects NSS to 14.
> R6 connects SCK to 15.
> R7 connects MOSI to 16.
> R8 connects MISO to 17.
> R9 connects NRST to 18.
> R10 connects NC to GND.
> R11 connects 10 to GND.
> C3 connects GND to NC.
> C4 connects GND to NC.
> C5 connects GND to NC.
> C6 connects GND to NC.
> C2 connects GND to 1.
> C1 connects GND to 2.
>
> Connections involving pins and signals:
> 9 (BUSY) connects to U1 BUSY.
> 7 (NSS) connects to U1 NSS.
> 6 (SCK) connects to U1 SCK.
> 5 (MOSI) connects to U1 MOSI.
> 19 (MISO) connects to U1 MISO.
> 3 (RESET) connects to U1 NRST.
> 1 (VCC_IN) connects to FB1.
> 2 (VCC_IN) connects to FB1.
> 10 (SUBG_RF) connects to R11.
> 11 (GND) connects to GND.
> 12 (GND) connects to GND.
> 13 (2.4G_RF) connects to GND.
> 14 (GND) connects to GND.
> 15 (GND) connects to GND
> (Description cut: the model started repeating itself.)

**Figure 11 Reference Design Based on Wio-LR2021**

**PCB Layout Notice:**

1\. SPI interface and DIOs between host MCU and Wio-LR2021 must be connected via a 470 Ω resistor to suppress high-frequency harmonics on signal lines. All 470 Ω resistors should be placed as close as possible to the Wio-LR2021 module.

**RF Trace Design Guidelines:**

1. The output impedance of the RF channels are both 50 Ω and compatible with IPEX and SMT connectors. 1. The RF trace impedance specifications are based on a standard 4-layer FR-4 PCB construction with 1 oz (35 µm) outer-layer copper and Prepreg 2116 (~0.10 mm / 4 mil). 2. To maintain a 50 Ω characteristic impedance (±10%) for the microstrip traces routing from the module's RF pinout to the IPEX/SMA connectors, a trace width of (0.152 mm / 6 mil) is recommended. 3. Place stitching vias at ≤1mm intervals along both sides of the RF trace to suppress parasitic coupling and ensure return path integrity.

### 5.3. Host Interface Timing

#### 5.3.1. SPI Timing Specification

The Wio-LR2021 module operates as an SPI subordinate device. The module SPI port conforms to Mode 0 (CPOL = 0, CPHA = 0). The host must configure its SPI controller accordingly. MISO is high-impedance when NSS is de-asserted (HIGH). The SPI runs on the external SCK clock with a maximum frequency of 16MHz.

![Figure 12 SPI Timing Diagram](images/p018-diagram-01.png)

> **Image description:** Timing diagram from Figure 12 SPI Timing Diagram.
>
> Signals from top to bottom are NSS, SCK, MOSI, and MISO.
>
> The sequence of edges and states in time order is as follows:
> NSS starts high and goes low, triggering tSLCH.
> SCK starts low and goes high, triggering tDIS.
> MOSI starts low and goes high, triggering tSLDO.
> MISO starts high and remains high when NSS is de-asserted (HIGH).
> SCK goes low, triggering tDIH.
> MOSI goes low, triggering tCLDO.
> SCK goes high, triggering tCH.
> MOSI goes high, triggering tCHDO.
> SCK goes low, triggering tCL.
> SCK goes high, triggering tCHSH.
> NSS goes high, triggering tSH.
>
> Named intervals are:
> tSLCH
> tCP
> tCH
> tCL
> tCHSH
> tSH
> tSLDO
> tCLDO
> tCHDO
> tDIS
> tDIH
>
> No specific values or units are shown for any named interval.

**Figure 12 SPI Timing Diagram**

### SPI Timing Requirements

<table>
<thead>
<tr><th>Symbol</th><th>Description</th><th>Min</th><th>Max</th><th>Unit</th></tr>
</thead>
<tbody>
<tr><td>t<sub>SH</sub></td><td>NSS high time (bus release)</td><td>125</td><td>-</td><td>ns</td></tr>
<tr><td>t<sub>SLCH</sub></td><td>NSS falling edge to firstSCK rising edge (setup)</td><td>31.25</td><td>-</td><td>ns</td></tr>
<tr><td>t<sub>CP</sub></td><td>SCK clock period</td><td>61.5</td><td>-</td><td>ns</td></tr>
<tr><td>t<sub>CH</sub></td><td>SCK high time</td><td>31.25</td><td>-</td><td>ns</td></tr>
<tr><td>t<sub>CL</sub></td><td>SCK low time</td><td>31.25</td><td>-</td><td>ns</td></tr>
<tr><td>t<sub>CHSH</sub></td><td>Last SCK falling edge to NSS rising edge (hold)</td><td>65</td><td>-</td><td>ns</td></tr>
<tr><td>t<sub>SLDO</sub></td><td>MISO valid delay from NSS falling edge</td><td>1</td><td>10</td><td>ns</td></tr>
<tr><td>t<sub>DIS</sub></td><td>MOSI data setup time</td><td>15</td><td>-</td><td>ns</td></tr>
<tr><td>t<sub>DIH</sub></td><td>MOSI data hold time</td><td>15</td><td>-</td><td>ns</td></tr>
<tr><td>t<sub>CLDO</sub></td><td>MISO valid delay from SCK falling edge</td><td>-</td><td>18</td><td>ns</td></tr>
<tr><td>t<sub>CHDO</sub></td><td>SCK rising edge to MISO data output time</td><td>-</td><td>10</td><td>ns</td></tr>
<tr><td colspan="5">NOTE:<br>1. The module SPI interface does not support continuous frame mode (NSS kept low across multiple opcodes). NSS must be released (HIGH) for t<sub>SH</sub> minimum between each command frame.</td></tr>
</tbody>
</table>

#### 5.3.2. BUSY Signal Specification

The BUSY signal (module pin 8) provides hardware flow control between the host and the module radio controller:

- When the BUSY line is held LOW, it indicates that the radio is ready to accept a command from the host controller.
- The BUSY pin will be automatically pulled HIGH on the falling edge of NSS.
- Once the chip has finished processing the command, the BUSY line will be released(LOW), indicating that the chip has entered a stable mode and is ready to accept another command.
- In FS mode, BUSY goes LOW when the PLL is locked.
- In Rx mode, BUSY goes LOW as soon as the chip is ready to receive data.
- In Tx mode, BUSY goes LOW when the PA has ramped-up and transmission of preamble starts.
- When triggered by a DIO pin, the BUSY signal will not be raised.

**NOTE:**

For exhaustive SPI timing parameters, command opcodes, and host interface protocol details, please refer to the Semtech LR2021 Datasheet. Documentation: Semtech LoRa Plus LR2021 Datasheet

The Semtech official RF driver for the LR2021 is open-source and maintained on GitHub. The repository provides the complete driver implementation, hardware abstraction layer (HAL), and reference applications to facilitate rapid development. Resource: Semtech Unified Software Platform(USP)

## 6. Production

### 6.1 Reflow Soldering Parameters

The Wio-LR2021 module is designed for easy integration into production processes, including soldering onto a PCB using reflow soldering techniques. A critical factor is that users must select the appropriate solder paste and ensure it meets the temperature requirements during the reflow process.

Note: During reflow soldering, the module temperature must not exceed 260°C, and the duration in the reflow zone should not exceed 30 seconds.

![Figure 13 Soldering Profile](images/p020-figure-01.png)

> **Image description:** Graph of Soldering Profile.
>
> X-axis: Temperature, unit is not specified, range is 0 to 360.
> Y-axis: Temperature, unit is not specified, range is 0 to 300.
>
> Curves:
> Target Curve: Starts low, rises to a peak around 180, then decreases.
> Minimum Preheat Temperature: Remains relatively flat across the temperature range shown.
> Maximum Preheat Temperature: Remains relatively flat across the temperature range shown.
> Liquefaction Threshold: Remains relatively flat across the temperature range shown.
> Low Point of Reflow Zone: Rises, peaks, and then decreases.
> High Point of Reflow Zone: Rises, peaks, and then decreases.
> Absolute Peak Value: Rises, peaks, and then decreases.
>
> Text inside the figure:
> x: 0 to 360
> y: 0 to 300
>
> Legend values:
> Target Curve
> Minimum Preheat Temperature
> Maximum Preheat Temperature
> Liquefaction Threshold
> Low Point of Reflow Zone
> High Point of Reflow Zone
> Absolute Peak Value
>
> Text that mentions the figure:
> Note: During reflow soldering, the module temperature must not exceed 260°C, and the duration in the reflow zone should not exceed 30 seconds.

**Figure 13 Soldering Profile**

| Item | Value | Unit |
| --- | --- | --- |
| Heating Rate | 1~3 | °C/s |
| Cooling Rate | 2~4 | °C/s |
| Heating Rate of Preheating Zone | 0.5~1 | °C/s |
| Minimum Time of Preheating Zone | 70 | s |
| Maximum Time of Preheating Zone | 120 | s |
| Minimum Temperature of Preheating Zone | 150 | °C |
| Maximum Temperature of Preheating Zone | 190 | °C |
| Minimum Residence Time of Solder Paste (above liquefaction temperature) | 50 | s |
| Maximum Residence Time of Solder Paste (above liquefaction temperature) | 70 | s |
| Residence Time in Reflux Zone | 30 | s |
| Maximum Residence Time at Peak Temperature | 5 | s |
| Recommand Liquefaction Zone Threshold | 218 | °C |
| Low Point Temperature of Reflux Zone | 240 | °C |
| High Point Temperature of Reflux Zone | 250 | °C |
| Peak Temperature | 260 | °C |

### 6.2 Tape & Reel

The Wio-LR2021 module can be supplied taped and reeled in quantities of 1236 per reel. The tape and reel packaging followsindustry standards with the following dimensions and specifications:

![Picture on page 21](images/p021-picture-01.png)

**Figure 14 Tape and Reel Dimension**

<table>
<thead>
<tr><th colspan="8"><b>Tape &amp; Reel Dimensions</b></th></tr>
</thead>
<tbody>
<tr><td>W</td><td>32.0±0.30</td><td>P</td><td>16.0±0.10</td><td>A</td><td>10.9±0.10</td><td>B0</td><td>17.6±0.10</td></tr>
<tr><td>SO</td><td>/</td><td>P0</td><td>4.0±0.10</td><td>A1</td><td></td><td>B1</td><td></td></tr>
<tr><td>E</td><td>1.75±0.10</td><td>P2</td><td>2.0±0.10</td><td>A2</td><td></td><td>B2</td><td></td></tr>
<tr><td>F</td><td>14.2±0.10</td><td>D0</td><td>Ø1.5</td><td>K0</td><td>3.45±0.10</td><td></td><td></td></tr>
<tr><td>T</td><td>0.4±0.05</td><td>D1</td><td></td><td>K1</td><td></td><td></td><td></td></tr>
</tbody>
</table>

NOTE:  
1 Dimensions comply with the FIA-481-D standard

2. All dimensions in millimeters. Cumulative tolerance for 10 sprocket hole pitches: ±0.20 mm. 3. Material: Conductive polystyrene.

## 7. Version

| V1.0 | 2026-05-14 | First Release | David Du |
| --- | --- | --- | --- |
| V1.1 | 2026-08-26 | Fix VTCXO Voltage | David Du |
