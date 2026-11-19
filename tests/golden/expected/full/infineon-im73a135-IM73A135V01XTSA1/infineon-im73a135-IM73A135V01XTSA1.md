# IM73A135V01

## IP57 dust and water resistant analog XENSIVTM MEMS microphone

## Description

The IM73A135V01 is designed for applications which require a microphone with high SNR (low self-noise), low distortion (high AOP), which is also IP57 robust to dust and water.

Best-in-class Signal to Noise Ratio (SNR) of 73dB(A) enables far field and low volume audio pick-up. The flat frequency response ( 20Hz low-frequency roll-off) and tight manufacturing tolerance improve the performance of multi-microphone array applications.

The high performance analog microphone ASIC contains an extremely low noise preamplifier and a high performance differential output amplifier. Different power modes can be selected in order to suit specific current consumption requirements.

Each IM73A135V01 microphone is calibrated with an advanced Infineon calibration algorithm, resulting in small sensitivity tolerances (± 1dB).

## Features

- Component level IP57 water and dust resistant
- Signal to noise ratio of 73dB(A) SNR
- Acoustic overload point at 135dBSPL
- Flat frequency response with low frequency roll off at 20Hz
  - Power optimized modes determined by V<sub>DD</sub> level
  - Package dimensions: 4mm x 3mm x 1.2mm
  - Differential output
  - Omnidirectional pickup pattern

## Typical applications

- Active Noise Cancellation (ANC) headphones and wireless earbuds
- Devices with Voice User Interface (VUI)
  - Smart speakers
  - Home automation
  - IoT devices
    - High quality audio capturing
    - Conference systems
    - Cameras and camcorders
    - Industrial or home monitoring with audio pattern detection

## Ordering information

**Table 1 Ordering information**

| <b>Product name</b> | <b>Package</b> | <b>Marking</b> | <b>Ordering code</b> |
| --- | --- | --- | --- |
| IM73A135V01 | PG-LLGA-5-2 | I73A03 | SP003803274 |

![Picture on page 1](images/p001-picture-01.png)

> **Image description:** Rendering of an electronic component package and its corresponding PCB footprint.
>
> The object on the left is a surface-mount microphone package. Visible markings on the package include the Infineon logo and the part number IM73A135.
>
> The object on the right is a PCB footprint for the component. It shows a gold-colored circular pad with a central hole, and four gold-colored rectangular pads arranged in a square pattern below the circle.
>
> No dimensions or scale are shown.

## Block diagram

![Figure 1 IM73A135V01 block diagram](images/p002-diagram-01.png)

> **Image description:** Block diagram of IM73A135V01.
>
> Blocks:
> MEMS
> ASIC
> CALIBRATION
> BAND GAP
> LDO
> POWER MODE DETECTOR
> AMP
> AMP
> VDD
> GROUND
> MEMS BIAS CHARGE PUMP
> MEMBRANE 1
> MEMBRANE 2
> STATOR
> OUT+
> OUT-
>
> Connections:
> MEMS connects to MEMBRANE 1, MEMBRANE 2, and STATOR.
> MEMS connects to ASIC.
> ASIC connects to CALIBRATION.
> ASIC connects to MEMS BIAS CHARGE PUMP.
> ASIC connects to BAND GAP.
> ASIC connects to LDO.
> ASIC connects to POWER MODE DETECTOR.
> ASIC connects to AMP (top).
> ASIC connects to AMP (bottom).
> VDD connects to CALIBRATION, BAND GAP, LDO, and POWER MODE DETECTOR.
> GROUND connects to ASIC.
> MEMS BIAS CHARGE PUMP connects to ASIC.
> CALIBRATION connects to ASIC.
> BAND GAP connects to ASIC.
> LDO connects to ASIC.
> POWER MODE DETECTOR connects to ASIC.
> AMP (top) connects to ASIC and OUT+.
> AMP (bottom) connects to ASIC and OUT-.
>
> Numbers:
> No voltage, frequency, part value, or other number with a unit is shown.

**Figure 1 IM73A135V01 block diagram**

## Product validation

Technology qualified for industrial applications.

Ready for validation in industrial applications according to the relevant tests of IEC 60747 and 60749 or alternatively JEDEC47/20/22.

## Environmental robustness

Infineon’s latest Sealed Dual Membrane MEMS technology delivers high ingress protection (IP57) at a microphone level. The sealed MEMS design prevents water or dust from entering between membrane and backplate, preventing mechanical blockage or electric leakage issues commonly observed in MEMS microphones. Microphones built with the sealed dual membrane technology can be used to create IP68 devices, requiring only minimal mesh protection.

**Table 2 Environmental robustness**

<sup>1</sup> The number "5" stands for the dust ingress rating or the capacity to withstand the effects of fine, abrasive dust particles.  
<sup>2</sup> The number "7" identifies the level of liquid protection.

| <b>Test Standard</b> | <b>Test Condition</b> |
| --- | --- |
| IP5x dust resistance<sup>1)</sup> | Arizona dust A4 coarse, vertical orientation , sound hole upwards, 10 cycles (15 minutes sedimentation, 6 sec blowing) |
| IPx7 water immersion<sup>2)</sup> | Temporary immersion in 1 meter of water for 30 minutes. Microphone tested 2 hours after removal |

## Table of contents

Description … 1  
Features … 1  
Typical applications … 1  
Ordering information … 1  
Block diagram … 2  
Product validation … 2  
Environmental robustness … 2  
Table of contents … 3  
1 Typical performance characteristics … 4  
2 Acoustic characteristics … 6  
2.1 Free field frequency response … 7  
3 Electrical characteristics and parameters … 8  
3.1 Absolute maximum ratings … 8  
3.2 Electrical parameters … 8  
3.3 Electrical characteristics … 10  
4 Package information … 11  
5 Footprint and stencil recommendation … 12  
6 Packing information … 13  
7 Reflow soldering and board assembly … 14  
8 Reliability specifications … 16  
Revision history … 17  
Disclaimer … 18

## 1 Typical performance characteristics

Test conditions(unless otherwise specified): V<sub>DD</sub> = 2.75V, T<sub>A</sub>= 25°C, output unloaded

![Figure 2 Typical amplitude response](images/p004-figure-04.png)

> **Image description:** Graph of Typical amplitude response.
>
> The horizontal axis plots Frequency [Hz] with a range from 10 to 10000. The vertical axis plots Normalized Output Level [dB] with a range from -12 to 12.
>
> There is one curve shown. This curve shows a general increasing trend in the Normalized Output Level as the Frequency increases, starting from a low level and rising towards the higher end of the frequency range.
>
> Text printed in the figure includes: x: Frequency [Hz], 10 to 10000 | y: Normalized Output Level [dB], -12 to 12. The test conditions mentioned are: VDD = 2.75V, TA= 25°C, output unloaded.

**Figure 2 Typical amplitude response**

![Figure 3 Typical free field ultrasonic response](images/p004-figure-01.png)

> **Image description:** Graph of Typical free field ultrasonic response.
>
> The x-axis plots Frequency [Hz] with a range from 0 to 80000.
> The y-axis plots Sensitivity relative to 1kHz [dB] with a range from -20 to 30.
>
> There is one curve shown. The overall trend of the curve starts low, increases to a peak around 30000 Hz, and then generally decreases before slightly increasing again towards the end of the measured range.
>
> Text printed in the figure includes:
> x: Frequency [Hz], 0 to 80000
> y: Sensitivity relative to 1kHz [dB], -20 to 30
> Test conditions(unless otherwise specified): VDD = 2.75V, TA= 25°C, output unloaded

**Figure 3 Typical free field ultrasonic response**

![Figure 4 Typical group delay and phase response](images/p004-figure-05.png)

> **Image description:** Graph of typical group delay and phase response.
>
> The horizontal axis plots Frequency [Hz] with a range from 10 to 10000.
> The left vertical axis plots Group Delay [s] with a range from 0.0000001 to 1.
> The right vertical axis plots Phase [°] with a range from -180 to 90.
>
> There are two curves:
> - The curve labeled Phase shows a general decreasing trend as frequency increases.
> - The curve labeled Group delay shows a general decreasing trend as frequency increases.
>
> Text printed in the figure includes:
> Phase
> Group delay
> x: Frequency [Hz], 10 to 10000
> y: Group Delay [s], 0.0000001 to 1
> y2: Phase [°], -180 to 90
>
> Test conditions are: VDD = 2.75V, TA= 25°C, output unloaded.

**Figure 4 Typical group delay and phase response**

![Figure 5 Typical noise floor (unweighted)](images/p004-figure-02.png)

> **Image description:** Graph of Spectral Noise Density versus Frequency.
> The vertical axis plots Spectral Noise Density in dBV/sqrt(Hz), ranging from -170 to -80.
> The horizontal axis plots Frequency in Hz, ranging from 10 to 10000.
>
> There are two curves:
> - Low power mode: This curve starts around -120 dBV/sqrt(Hz) at 10 Hz and generally decreases, leveling off towards the right side of the graph.
> - Normal mode: This curve starts around -125 dBV/sqrt(Hz) at 10 Hz and generally decreases, leveling off towards the right side of the graph, remaining below the Low power mode curve.
>
> Text printed in the figure includes:
> - Low power mode
> - Normal mode
> - x: Frequency [Hz], 10 to 10000
> - y: Spectral Noise Density [dBV/sqrt(Hz)], -170 to -80
>
> Test conditions mentioned are: VDD = 2.75V, TA= 25°C, output unloaded.

**Figure 5 Typical noise floor (unweighted)**

![Figure 6 Typical THD vs SPL](images/p004-figure-06.png)

> **Image description:** Graph of Typical THD vs SPL.
>
> The horizontal axis plots Input Sound Pressure Level [dB] with a range from 90 to 140.
> The vertical axis plots THD [%] with a range from 0.01 to 10.
>
> There are two curves:
> - Low power mode: This curve shows a generally increasing trend in THD as the Input Sound Pressure Level increases.
> - Normal mode: This curve also shows a generally increasing trend in THD as the Input Sound Pressure Level increases, and it is positioned above the Low power mode curve for most of the measured range.
>
> Text printed in the figure includes:
> - Low power mode
> - Normal mode
> - x: Input Sound Pressure Level [dB], 90 to 140
> - y: THD [%], 0.01 to 10
>
> Test conditions mentioned are: VDD = 2.75V, TA= 25°C, output unloaded.

**Figure 6 Typical THD vs SPL**

![Figure 7 Typical THD vs frequency](images/p004-figure-03.png)

> **Image description:** Graph of Typical THD vs frequency.
>
> The x-axis plots Frequency [Hz] with a range from 100 to 10000.
> The y-axis plots THD [%] with a range from 0.01 to 10.
>
> There are three curves:
> - The curve labeled 135dB SPL shows a generally increasing trend across the frequency range.
> - The curve labeled 133dB SPL shows a generally increasing trend across the frequency range.
> - The curve labeled 110dB SPL shows a generally increasing trend across the frequency range.
>
> Text printed in the figure includes:
> - x: Frequency [Hz], 100 to 10000
> - y: THD [%], 0.01 to 10
> - 135dB SPL
> - 133dB SPL
> - 110dB SPL
>
> Test conditions(unless otherwise specified): VDD = 2.75V, TA= 25°C, output unloaded.

**Figure 7 Typical THD vs frequency**

![Figure 8 Typical IDD vs VDD](images/p005-figure-02.png)

> **Image description:** Graph of typical IDD vs VDD.
>
> The x-axis plots V_DD [V] with a range from 1.60 to 3.60.
> The y-axis plots I_DD [uA] with a range from 0 to 200.
>
> There are two curves:
> - Normal mode, SNR=73dB: This curve shows an increasing trend as V_DD increases.
> - Low power mode, SNR=71dB: This curve shows an increasing trend as V_DD increases.
>
> Text printed in the figure includes:
> Normal mode, SNR=73dB
> Low power mode, SNR=71dB
> x: V_DD [V], 1.60 to 3.60
> y: I_DD [uA], 0 to 200

**Figure 8 Typical I<sub>DD</sub> vs V<sub>DD</sub>**

![Figure 9 Typical PSRR](images/p005-figure-01.png)

> **Image description:** Graph of Typical PSRR.
>
> The horizontal axis plots Frequency [Hz] with a range from 10 to 10000. The vertical axis plots PSRR [dB] with a range from 40 to 110.
>
> There are two curves:
> - Low power mode: This curve starts around 70 dB at 10 Hz, increases to a peak near 100 dB around 1000 Hz, and then gradually decreases towards 90 dB at 10000 Hz.
> - Normal mode: This curve starts around 65 dB at 10 Hz, increases to a peak near 105 dB around 1000 Hz, and then gradually decreases towards 95 dB at 10000 Hz.
>
> Text printed in the figure includes:
> - Low power mode
> - Normal mode
> - x: Frequency [Hz], 10 to 10000
> - y: PSRR [dB], 40 to 110

**Figure 9 Typical PSRR**

## 2 Acoustic characteristics

Test conditions (unless otherwise specified in the table): V<sub>DD</sub> = 2.75V, T<sub>A</sub> = 25°C, 55% R.H., audio bandwidth 20Hz to 20kHz, output unloaded

**Table 3 IM73A135V01 acoustic specifications**

<table>
<thead>
<tr><th colspan="2" rowspan="2">Parameter</th><th rowspan="2">Symbol</th><th colspan="3">Values</th><th rowspan="2">Unit</th><th rowspan="2">Note or Test Condition</th></tr>
<tr><th>Min.</th><th>Typ.</th><th>Max.</th></tr>
</thead>
<tbody>
<tr><td colspan="2">Sensitivity</td><td></td><td>-39</td><td>-38</td><td>-37</td><td>dBV</td><td>1kHz, 94 dB SPL, all operating modes</td></tr>
<tr><td colspan="3">Low Frequency Cutoff Point</td><td>f<sub>CLP</sub></td><td></td><td>20</td><td></td><td>Hz</td><td>-3dB point relative to 1kHz</td></tr>
<tr><td rowspan="2">Acoustic overload point</td><td colspan="2">Normal mode</td><td>AOP<sub>NP</sub></td><td></td><td>135</td><td></td><td rowspan="2">dBSPL</td><td rowspan="2">THD = 10%.</td></tr>
<tr><td colspan="2">Low power mode</td><td>AOP<sub>LP</sub></td><td></td><td>130</td><td></td></tr>
<tr><td rowspan="2">Signal to Noise ratio</td><td colspan="2">Normal mode</td><td>SNR<sub>NP</sub></td><td></td><td>73</td><td></td><td rowspan="2">dB(A)</td><td rowspan="2">A-Weighted</td></tr>
<tr><td colspan="2">Low power mode</td><td>SNR<sub>LP</sub></td><td></td><td>71</td><td></td></tr>
<tr><td rowspan="2">Noise floor</td><td colspan="2">Normal mode</td><td rowspan="2"></td><td></td><td>-111</td><td></td><td rowspan="2">dBV(A)</td><td rowspan="2">A-Weighted</td></tr>
<tr><td colspan="3">Low power mode</td><td></td><td>-109</td><td></td></tr>
<tr><td rowspan="6">Total harmonic distortion</td><td rowspan="3">Normal mode</td><td>94dBSPL</td><td rowspan="3">THD<sub>NP</sub></td><td></td><td>0.5</td><td rowspan="6">%</td><td rowspan="6">Measuring 2nd to 5th harmonics, 1kHz</td></tr>
<tr><td>132dBSPL</td><td></td><td>1.0</td></tr>
<tr><td>135dBSPL</td><td></td><td>10.0</td></tr>
<tr><td rowspan="3">Low power mode</td><td>94dBSPL</td><td rowspan="3">THD<sub>LP</sub></td><td></td><td>0.5</td></tr>
<tr><td>124dBSPL</td><td></td><td>1.0</td></tr>
<tr><td>130dBSPL</td><td></td><td>10.0</td></tr>
<tr><td rowspan="3">Phase Response</td><td colspan="2">75Hz</td><td></td><td></td><td>12</td><td rowspan="3">°</td><td rowspan="3"></td></tr>
<tr><td colspan="2">1kHz</td><td></td><td></td><td>2</td></tr>
<tr><td colspan="2">3kHz</td><td></td><td></td><td>-2</td></tr>
<tr><td rowspan="4">Group Delay</td><td colspan="2">250Hz</td><td></td><td></td><td>52</td><td rowspan="4">µs</td><td rowspan="4"></td></tr>
<tr><td colspan="2">600Hz</td><td></td><td></td><td>7</td></tr>
<tr><td colspan="2">1kHz</td><td></td><td></td><td>2</td></tr>
<tr><td colspan="2" rowspan="3">4kHz</td><td></td><td></td><td>0.5</td></tr>
<tr><td colspan="3">Directivity</td><td colspan="3">Omnidirectional</td><td></td><td>Pickup pattern</td></tr>
<tr><td colspan="3">Polarity</td><td colspan="3">Increasing Vout</td><td></td><td>Increasing SPL</td></tr>
</tbody>
</table>

### 2.1 Free field frequency response

![Figure 10 IM73A135V01 free field frequency response](images/p007-figure-01.png)

> **Image description:** Graph of free field frequency response.
> The x-axis plots Frequency [Hz] with a range from 10 to 10000.
> The y-axis plots Normalized output level [dB] with a range from -12 to 12.
>
> There are three curves shown:
> - Upper limit: This curve generally increases across the frequency range, starting around -1 dB at 10 Hz and rising to approximately 9 dB at 10000 Hz.
> - Lower limit: This curve generally increases across the frequency range, starting around -4 dB at 10 Hz and rising to approximately 1 dB at 10000 Hz.
> - A solid black curve: This curve remains relatively flat across the lower frequencies, hovering near 0 dB, and then shows a gradual increase towards the higher frequencies, reaching approximately 3 dB at 10000 Hz.
>
> Text printed in the figure includes:
> - x: Frequency [Hz], 10 to 10000
> - y: Normalized output level [dB], -12 to 12
> - Legend entries: Upper limit, Lower limit.

**Figure 10 IM73A135V01 free field frequency response**

**Table 4 IM73A135V01 free field frequency response, normalised to 1kHz sensitivity value.**

| <b>Frequency (Hz)</b> | <b>Upper Limit (dB)</b> | <b>Lower Limit (dB)</b> |
| --- | --- | --- |
| 20 | -1.5 | -4.5 |
| 50 | +0.5 | -1.5 |
| 800 | +1 | -1 |
| 1000 | 0 | 0 |
| 1200 | +1 | -1 |
| 6000 | +2 | -1 |
| 8000 | +4 | -1 |
| 15000 | +9 | 0 |

## 3 Electrical characteristics and parameters

### 3.1 Absolute maximum ratings

Stresses at or above the listed maximum ratings may affect device reliability or cause permanent device damage. Functional device operation at these conditions is not guaranteed.

**Table 5 Absolute maximum ratings**

### 3.2 Electrical parameters

**Table 6 IM73A135V01 electrical parameters**

<table>
<thead>
<tr><th rowspan="2">Parameter</th><th rowspan="2">Symbol</th><th colspan="2">Values</th><th rowspan="2">Unit</th><th rowspan="2">Note / Test Condition</th></tr>
<tr><th>Min.</th><th>Max.</th></tr>
</thead>
<tbody>
<tr><td>Supply voltage</td><td>V<sub>DDmax</sub></td><td></td><td>3.0</td><td>V</td><td></td></tr>
<tr><td>Storage temperature</td><td>T<sub>S</sub></td><td>-40</td><td>100</td><td>°C</td><td></td></tr>
<tr><td>Operating temperature</td><td>T<sub>A</sub></td><td>-40</td><td>85</td><td>°C</td><td></td></tr>
</tbody>
</table>

<table>
<thead>
<tr><th colspan="2" rowspan="2">Parameter</th><th rowspan="2">Symbol</th><th colspan="3">Values</th><th rowspan="2">Unit</th><th rowspan="2">Note / Test Condition</th></tr>
<tr><th>Min.</th><th>Typ.</th><th>Max.</th></tr>
</thead>
<tbody>
<tr><td rowspan="2">Supply voltage</td><td>Normal mode</td><td rowspan="2">VDD</td><td>2.3</td><td>2.75</td><td>3.0</td><td rowspan="2">V</td><td rowspan="2">A 100nF bypass capacitor(C<sub>VDD</sub>) should be placed close to the microphone V<sub>DD</sub> pin to ensure best SNR performance</td></tr>
<tr><td>Low power mode</td><td>1.52</td><td>1.6</td><td>1.8</td></tr>
<tr><td colspan="2">V<sub>DD</sub> ramp-up time</td><td></td><td></td><td></td><td>5</td><td>ms</td><td>V<sub>DD</sub> reaches its final value within ± 10 % tolerance</td></tr>
<tr><td rowspan="3">Output load</td><td rowspan="2">C<sub>load</sub></td><td>C<sub>a</sub>, C<sub>b</sub></td><td></td><td rowspan="3">25</td><td>100</td><td rowspan="2">pF</td><td rowspan="3"></td></tr>
<tr><td>C<sub>d</sub></td><td></td><td>100</td></tr>
<tr><td>R<sub>load</sub></td><td>R<sub>p</sub>, R<sub>n</sub></td><td></td><td></td><td>kΩ</td></tr>
<tr><td rowspan="2">Input VCM</td><td>Normal mode</td><td rowspan="2">V<sub>cm</sub></td><td>1.17</td><td>1.3</td><td>1.43</td><td rowspan="2">V</td><td rowspan="2">Input VCM is the voltage at the input pins of the audio front end if not driven</td></tr>
<tr><td>Low power mode</td><td>0.765</td><td>0.85</td><td>0.935</td></tr>
</tbody>
</table>

![Figure 11 Output load configuration](images/p009-diagram-01.png)

> **Image description:** Schematic of the output load configuration for a microphone.
>
> Components:
> CVDD
> GND
> MIC
> VDD
> OUT+
> OUT-
> Ca
> Cb
> Cd
> Rp
> Rn
> VCM
>
> Connections:
> CVDD connects to GND.
> VDD connects to the top of the MIC block.
> The MIC block has connections to VDD, OUT+, OUT-, and GND.
> OUT+ connects to Ca.
> OUT- connects to Cb.
> Ca connects to Cd.
> Cd connects to Rp.
> Rp connects to Rn.
> Rn connects to VCM.
> GND connects to the bottom of the MIC block.
>
> Values:
> No specific voltage, frequency, or part values with units are shown, other than the labels for the components and pins.

**Figure 11 Output load configuration**

### 3.3 Electrical characteristics

Test conditions (unless otherwise specified in the table): V<sub>DD</sub> = 2.75V, T<sub>A</sub> = 25°C, 55% R.H.

**Table 7 IM73A135V01 electrical characteristics**

<table>
<thead>
<tr><th colspan="2" rowspan="2">Parameter</th><th rowspan="2">Symbol</th><th colspan="3">Values</th><th rowspan="2">Unit</th><th rowspan="2">Note / Test Condition</th></tr>
<tr><th>Min.</th><th>Typ.</th><th>Max.</th></tr>
</thead>
<tbody>
<tr><td rowspan="2">Current consumption</td><td>Normal mode</td><td>I<sub>DDNP</sub></td><td></td><td>170</td><td>230</td><td rowspan="2">µA</td><td rowspan="2">Input ≤ 94 dBSPL</td></tr>
<tr><td>Low power mode</td><td>I<sub>DDLP</sub></td><td></td><td>70</td><td>80</td></tr>
<tr><td colspan="2">Start-up time</td><td>t<sub>startup</sub></td><td></td><td>10</td><td>30</td><td>ms</td><td>Start-up time in all operating modes after V<sub>DD<sub>min</sub></sub> is applied</td></tr>
<tr><td colspan="2">Mode switching time</td><td>t<sub>ModeChange</sub></td><td></td><td>10</td><td></td><td>ms</td><td>Time of undefined output after mode change detected</td></tr>
<tr><td colspan="2">Brown out voltage</td><td>V<sub>BrownOut</sub></td><td></td><td>1.2</td><td></td><td>V</td><td>Brown out is triggered for voltage below V<sub>BrownOut</sub></td></tr>
<tr><td rowspan="2">Vout DC-voltage</td><td>Normal mode</td><td>V<sub>OUT_DC_NP</sub></td><td></td><td>1.35</td><td></td><td rowspan="2">V</td><td></td></tr>
<tr><td>Low power mode</td><td>V<sub>OUT_DC_LP</sub></td><td></td><td>0.9</td><td></td><td></td></tr>
<tr><td rowspan="2">Power supply rejection ratio in band (differential)</td><td>Normal mode</td><td>PSRR<sub>InBand_NP</sub></td><td></td><td>80</td><td></td><td rowspan="2">dB</td><td>V<sub>DD</sub>=2.75V + 100mV<sub>pp</sub> sinewave (PSR=1/Apower; Apower = Transfer function V<sub>DD</sub> → Differential Output of ASIC)</td></tr>
<tr><td>Low power mode</td><td>PSRR<sub>InBand_LP</sub></td><td></td><td>80</td><td></td><td>V<sub>DD</sub>=1.6V + 100mV<sub>pp</sub> sinewave (PSR=1/Apower; Apower = Transfer function V<sub>DD</sub> → Differential Output of ASIC)</td></tr>
<tr><td rowspan="2">Power supply rejection ratio common mode</td><td>Normal mode</td><td>PSRR<sub>CM_NP</sub></td><td></td><td>65</td><td></td><td rowspan="2">dB</td><td>V<sub>DD</sub>= 2.75 V + 100mV<sub>pp</sub> sinewave</td></tr>
<tr><td>Low power mode</td><td>PSRR<sub>CM_LP</sub></td><td></td><td>60</td><td></td><td>V<sub>DD</sub>= 1.6 V + 100mV<sub>pp</sub> sinewave</td></tr>
<tr><td rowspan="2">Output impedance</td><td>Normal mode</td><td>Z<sub>out_NP</sub></td><td></td><td></td><td>250</td><td>Ω</td><td></td></tr>
<tr><td>Low power mode</td><td>Z<sub>out_LP</sub></td><td></td><td></td><td>500</td><td>Ω</td><td></td></tr>
</tbody>
</table>

## 4 Package information

![Figure 12 IM73A135V01 package drawing](images/p011-diagram-01.png)

> **Image description:** Package drawing, Bottom view. Pin 1 is indicated by "Pin1 marking".
>
> Pins:
> 1: (No name provided)
> 2: (No name provided)
> 3: (No name provided)
> 4: (No name provided)
> 5: (No name provided)
>
> Dimensions:
> 2x 0.1C
> 3±0.08 (2.8)
> A
> 4±0.08 (3.8)
> 2x 0.1C
> C
> 0.25±0.04
> 1.28 Max.
> Bottom view
> 4x 0.7±0.05
> 0.1M ABC
> 0.05M C
> 1.7
> 0.85
> 0.3±0.1
> 0.7
> 1
> 3
> 2
> 4
> 0.4±0.05
> 1.48 (0.8)
> 5
> (1.32)
> (0.68)
> 0.6±0.05
> 1.2±0.05
> 2±0.05

**Figure 12 IM73A135V01 package drawing**

**Table 8 IM73A135V01 pin configuration**

| <b>Pin Number</b> | <b>Name</b> | <b>Description</b> |
| --- | --- | --- |
| 1 | Output + | Differential Output + |
| 2 | V<sub>DD</sub> | Power supply |
| 3 | Output - | Differential Output - |
| 4 | GND | Ground |
| 5 | GND | Ground |

## 5 Footprint and stencil recommendation

The acoustic port hole diameter in the PCB should be larger than the acoustic port hole diameter of the MEMS Microphone to ensure optimal performance. A PCB sound port size of radius 0.4 mm (diameter 0.8mm) is recommended.

The board pad and stencil aperture recommendations shown in **Figure 13** are based on Solder Mask Defined (SMD) pads. The specific design rules of the board manufacturer should be considered for individual design optimizations or adaptations.

![Figure 13 IM73A135V01 footprint and stencil recommendation](images/p012-diagram-01.png)

> **Image description:** Package or module: Top view of the footprint and stencil recommendation. Pin1 is indicated on the left side of the main body.
>
> Pins:
> Pin1: Pin1
>
> Dimensions:
> R0.4
> R1.04
> R0.58
> 0.68
> 0.8
> 0.45 4x
> 0.7 2x
> 0.85
> 0.75 4x
> 1.7
> R1 6x
> 0.12 6x
> (52°)
> R0.6
> 0.4
> 0.65 4x
> 0.8
> 0.4
> 0.85
> 1.7
>
> Legend:
> copper
> solder mask
> stencil apertures
>
> Notes:
> All undimensioned radii is 0.2mm
> All dimensions are in units mm

**Figure 13 IM73A135V01 footprint and stencil recommendation**

## 6 Packing information

For shipping and assembly the Infineon microphones are packed in product specific tape-and-reel carriers. A detailed drawing of the carrier can be seen in **Figure 14**.

![Figure 14 IM73A135V01 tape and reel packing information](images/p013-diagram-01.png)

> **Image description:** Package drawing of the module, top view. Pin1 marking is indicated on the left side of the module.
>
> The pins are not explicitly numbered in the provided drawing, only the Pin1 marking is shown.
>
> Dimensions:
> 8 (unit mm)
> 4 (unit mm)
> 0.25 (unit mm)
> 12 (unit mm)
> 4.3 (unit mm)
> 3.3 (unit mm)
> 1.4 (unit mm)
>
> Text:
> All dimensions are in units mm
> The drawing is in compliance with ISO 128-30, Projection Method 1 [ ]
> Drawing according to ISO 8015

**Figure 14 IM73A135V01 tape and reel packing information**

## 7 Reflow soldering and board assembly

Infineon MEMS microphones are qualified in accordance with the IPC/JEDEC J-STD-020D-01. The moisture sensitivity level of MEMS microphones is rated as MSL1. For PCB assembly of the MEMS microphone the widely used reflow soldering using a forced convection oven is recommended.

The soldering profile should be in accordance with the recommendations of the solder paste manufacturer to reach an optimal solder joint quality. The reflow profile shown in **Figure 15** is recommended for board manufacturing with Infineon MEMS microphones.

![Figure 15 Recommended reflow profile](images/p014-figure-01.png)

> **Image description:** Graph of Temperature versus Time.
> The vertical axis is Temperature, with units not specified, ranging from 25 to Tp.
> The horizontal axis is Time, with units not specified, ranging from unreadable to unreadable.
>
> The graph shows several temperature profiles over time.
> There is a preheat phase labeled Preheat, which starts at Tsmin and rises to Tsmax.
> Following the preheat, there is a ramp-up phase leading to the peak temperature Tp.
> The peak temperature is labeled Tp.
> After the peak, there is a ramp-down phase.
> A region between TL and Tp is labeled Critical Zone.
> There are labels Tsmin, Tsmax, Tp, TL, and Tp.
> There are also labels ts, t_l, and t_p.
> The text inside the figure includes: Tp, Critical Zone, TL to Tp, Tsmax, Tsmin, ts, Preheat, t_l, Ramp-up, Ramp-down, and t_p.

**Figure 15 Recommended reflow profile**

**Table 9 Reflow profile limits**

Note: For further information please consult the 'General recommendation for assembly of Infineon packages' document which is available on the Infineon Technologies **web page**

<sup>3</sup> Tolerance for peak profile temperature (T<sub>p</sub>) is defined as a supplier minimum and a user maximum

| <b>Profile feature</b> | <b>Pb-Free assembly</b> | <b>Sn-Pb Eutectic assembly</b> |
| --- | --- | --- |
| Temperature Min (T<sub>smin</sub>) | 150 °C | 100 °C |
| Temperature Max (T<sub>smax</sub>) | 200 °C | 150 °C |
| Time (T<sub>smin</sub> to T<sub>smax</sub>) (t<sub>s</sub>) | 60-120 seconds | 60-120 seconds |
| Ramp-up rate (T<sub>L</sub> to T<sub>P</sub>) | 3 °C/second max. | 3 °C/second max. |
| Liquidous temperature (T<sub>L</sub>) | 217 °C | 183 °C |
| Time (t<sub>L</sub>) maintained above T<sub>L</sub> | 60-150 seconds | 60-150 seconds |
| Peak Temperature (T<sub>P</sub>) | 260°C +0°C/-5°C | 235°C +0°C/-5°C |
| Time within 5°C of actual peak temperature (tp) <sup>3)</sup> | 20-40 seconds | 10-30 seconds |
| Ramp-down rate | 6 °C/second max. | 6 °C/second max. |
| Time 25°C to peak temperature | 8 minutes max. | 6 minutes max. |

The MEMS microphones can be handled using industry standard pick and place equipment. Care should be taken to avoid damage to the microphone structure as follows:

- Do not pick the microphone with vacuum tools which make contact with the microphone acoustic port hole.
- The microphone acoustic port hole should not be exposed to vacuum, this can destroy or damage the MEMS.
- Do not blow air into the microphone acoustic port hole. If an air blow cleaning process is used, the port hole must be sealed to prevent particle contamination.
- It is recommended to perform the PCB assembly in a clean room environment in order to avoid microphone contamination.
- Air blow and ultrasonic cleaning procedures shall not be applied to MEMS Microphones. A no-clean paste is recommended for the assembly to avoid subsequent cleaning steps. The microphone MEMS can be severely damaged by cleaning substances.
- To prevent the blocking or partial blocking of the sound port during PCB assembly, it is recommended to cover the sound port with protective tape during PCB sawing or system assembly.
- Do not use excessive force to place the microphone on the PCB. The use of industry standard pick and place tools is recommended in order to limit the mechanical force exerted on the package.

## 8 Reliability specifications

The microphone sensitivity after stress must deviate by no more than 3dB from the initial value.

**Table 10 Reliability specification**

<sup>4</sup> The microphone sensitivity must deviate by no more than 1dB from the initial value after 3 reflow cycles.

<table>
<thead>
<tr><th><b>Table 10</b> Reliability specification</th></tr>
<tr><th><b>Test</b></th><th><b>Abbreviation</b></th><th><b>Test Condition</b></th><th><b>Standard</b></th></tr>
</thead>
<tbody>
<tr><td>Low Temperature Operating Life</td><td>LTOL</td><td>T<sub>a</sub>=-40°C, VDD=3.6V, 1000 hours</td><td>JESD22-A108</td></tr>
<tr><td>Low Temperature Storage Life</td><td>LTSL</td><td>T<sub>a</sub>=-40°C, 1000 hours</td><td>JESD22-A119</td></tr>
<tr><td>High Temperature Operation Life</td><td>HTOL</td><td>T<sub>a</sub>=+125°C, VDD=3.6V, 1000 hours</td><td>JESD22-A108</td></tr>
<tr><td>High Temperature Storage Life</td><td>HTSL</td><td>T<sub>a</sub>=+125°C, 1000 hours</td><td>JESD22-A103</td></tr>
<tr><td rowspan="2">Temperature Cycling</td><td rowspan="2">PC + TC</td><td>Pre conditioning MSL-1</td><td>JESD22-A113</td></tr>
<tr><td>1000 cycles, -40°C to +125°C, 30 minutes per cycle</td><td>JESD22-A104</td></tr>
<tr><td rowspan="2">Temperature Humidity Bias</td><td rowspan="2">PC + THB</td><td>Pre conditioning MSL-1</td><td>JESD22-A113</td></tr>
<tr><td>T<sub>a</sub>=+85°C, R.H = 85%, VDD=3.6V, 1000 hours</td><td>JESD22-A101</td></tr>
<tr><td>Vibration Test</td><td>VVF</td><td>20Hz to 2000Hz with a peak acceleration of 20g in X, Y, and Z for 4 minutes each, total 4 -cycles</td><td>IEC 60068-2-6</td></tr>
<tr><td>Mechanical Shock</td><td>MS</td><td>10000g/0.1msec direction ±x,y,z, 5 shocks in each direction, 5 shocks in total</td><td>IEC 60068-2-27</td></tr>
<tr><td>Reflow Solder<sup>4)</sup></td><td>RS</td><td>3 reflow cycles, peak temperature = +260°C</td><td>IPC-JEDEC J-STD-020D-01</td></tr>
<tr><td>Electrostatic Discharge -System Level Test</td><td>ESD - SLT</td><td>3 discharges of ±8kV direct contact to lid while V<sub>dd</sub> is supplied according to the operational modes; (V<sub>dd</sub> ground is separated from earth ground)</td><td>IEC-61000-4-2</td></tr>
<tr><td>Electrostatic Discharge - Human Body Model</td><td>ESD - HBM</td><td>1 pulse of ±2kV between all I/O pin combinations</td><td>JEDEC-JS001</td></tr>
<tr><td>Electrostatic Discharge - Charged Device Model</td><td>ESD - CDM</td><td>3 discharges of ±500V direct contact to I/O pins.</td><td>JEDEC JS-002</td></tr>
<tr><td>Latch up</td><td>LU</td><td>Trigger current from ±200mA</td><td>JESD78</td></tr>
</tbody>
</table>

## Revision history

| <b>Document version</b> | <b>Date of release</b> | <b>Description of changes</b> |
| --- | --- | --- |
| V 1.00 | 2021-01-07 | Initial release |
| V 1.10 | 2021-04-19 | Updated <b><i>Typical performance characteristics</i></b> |
| V 1.20 | 2021-07-07 | Updated <b><i>Figure 11</i></b> title |

## Trademarks

All referenced product or service names and trademarks are the property of their respective owners.

## Edition 2021-07-07 Published by Infineon Technologies AG 81726 Munich, Germany

**© 2021 Infineon Technologies AG All Rights Reserved.**

**Do you have a question about any aspect of this document?**  
**Email: erratum@infineon.com**

## Document reference IFX-mcx1539678668960

## IMPORTANT NOTICE

The information given in this document shall in no event be regarded as a guarantee of conditions or characteristics (“Beschaffenheitsgarantie”).

With respect to any examples, hints or any typical values stated herein and/or any information regarding the application of the product, Infineon Technologies hereby disclaims any and all warranties and liabilities of any kind, including without limitation warranties of non-infringement of intellectual property rights of any third party.

In addition, any information given in this document is subject to customer’s compliance with its obligations stated in this document and any applicable legal requirements, norms and standards concerning customer’s products and any use of the product of Infineon Technologies in customer’s applications.

The data contained in this document is exclusively intended for technically trained staff. It is the responsibility of customer’s technical departments to evaluate the suitability of the product for the intended application and the completeness of the product information given in this document with respect to such application.

## WARNINGS

Due to technical requirements products may contain dangerous substances. For information on the types in question please contact your nearest Infineon Technologies office.

Except as otherwise explicitly approved by Infineon Technologies in a written document signed by authorized representatives of Infineon Technologies, Infineon Technologies’ products may not be used in any applications where a failure of the product or any consequences of the use thereof can reasonably be expected to result in personal injury.
