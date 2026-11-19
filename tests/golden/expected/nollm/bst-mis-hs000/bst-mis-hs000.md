![Picture on page 1](images/p001-picture-01.png)

## HSMI for BMI2xy/BMI3xy/BMI4xy/BHI3xy IMUs

Document revision 1.7

Document release date August 2026

Document number BST-MIS-HS001-09

Sales Part Number(s)

0 273 017 000, 0 273 017 001, 0 273 017 008, 0 273 017 010, 0 273 017 056,

0 273 017 018, 0 273 017 028, 0 273 017 029, 0 273 017 037, 0 273 017 039, 0 273 017 061, 0 273 017 063, 0 273 017 064, 0 273 017 066, 0 273 017 068, 0 273 017 083, 0 273 017 084, 0 273 017 085, 0 273 017 086, 0 273 017 095

Notes Data and descriptions in this document are subject to change without notice. Product photos and pictures are for illustration purposes only and may differ from the real product appearance. The technical details and legal disclaimer of the respective product data sheet apply.

# Bosch Sensortec Inertial Measurement Units Handling, Soldering and Mounting Instructions

![Picture on page 1](images/p001-picture-02.png)

> **Image description:** Rendering of four electronic sensor modules.
>
> Top left module:
> - Object: Sensor module, viewed from the top.
> - Markings: BMI 260, BOSCH.
>
> Top right module:
> - Object: Sensor module, viewed from the top.
> - Markings: BMI 323, BOSCH.
>
> Bottom left module:
> - Object: Sensor module, viewed from the top.
> - Markings: BMI 430, BOSCH.
>
> Bottom right module:
> - Object: Sensor module, viewed from the top.
> - Markings: BHI 360, BOSCH.
>
> No dimensions or scale are shown.

## Table of Contents

1 Purpose of this document … 5  
2 Package outline … 5  
3 Landing pattern … 5  
4 Moisture sensitivity level (MSL) … 5  
5 RoHS compliance/halogen content … 5  
6 Mounting recommendations … 6  
6.1 External loads/forces … 6  
6.2 Thermal hot spots on the PCB … 7  
6.3 Mechanical stress maximum on the PCB … 8  
6.4 Distance to PCB anchor points … 8  
6.5 Vibrations on the PCB … 9  
6.5.1 Mechanical vibration sources … 9  
6.5.2 Electrical sources … 9  
6.5.3 MLCCs … 9  
6.5.4 PCB with large span … 11  
6.6 Resin coatings and shields … 11  
6.6.1 Encapsulation/over-molding/resin coatings … 11  
6.6.2 Shields … 12  
6.7 PCB precautions … 12  
6.8 Multiple IMUs on the same PCB … 13  
6.9 Magnetic, Electric, Infrared (IR), and Radio Frequency (RF) … 14  
6.10 Electrostatic charging … 14  
7 PCB Design Guidelines … 14  
7.1 Footprint design rules … 15  
7.2 Stencil design … 15  
7.3 PCB design rules for the traces … 16  
7.4 BMI2xy, BMI3xy and BMI4xy placement rules for the top layer … 17  
7.5 BHI3xy placement rules for the top layer … 17

Bosch Sensortec | HSMI for BMI2xy/BMI3xy/BMI4xy/BHI3xy IMUs 3 | 25  
7.6 Placement rules for the middle and bottom layers … 18  
8 Note on internal package structures … 18  
9 Device marking … 18  
10 Pick and place guidelines … 18  
11 Reflow soldering … 19  
11.1 Recommendation for soldering of sensors in the LGA package … 19  
11.2 Classification reflow profiles … 19  
11.3 Multiple reflow soldering cycles … 20  
12 Desoldering recommendations … 20  
13 Customer Return … 20  
14 Tape & reel … 21  
14.1 Tape & reel specification … 21  
14.2 Orientation within the reel … 21  
15 Further important mounting, assembly & handling recommendations … 22  
16 Document history and modification … 23

## List of figures

Figure 1: External loads/forces … 6  
Figure 2: Thermal hot spots on the PCB … 7  
Figure 3: Heat sinks … 7  
Figure 4: Mechanical stress maximum on the PCB … 8  
Figure 5: Distance to PCB anchor points … 8  
Figure 6: Deformation of MLCC affected by electric field … 9  
Figure 7: Placement of MLCCs to significantly reduce vibration … 10  
Figure 8: Long hanging beam or large span PCB … 11  
Figure 9: Encapsulation/over-molding/resin coatings fully cover the sensor … 11  
Figure 10: Resin coatings partially cover the sensor … 12  
Figure 11: Shields … 12  
Figure 12: Edge of PCB … 13  
Figure 13: PCB cut slot for multiple IMUs on the same PCB … 13  
Figure 14: NSMD and SMD … 14  
Figure 15: Footprint design rules … 15  
Figure 16: Stencil opening dimensions … 15  
Figure 17: Correct traces … 16  
Figure 18: High-speed traces … 16  
Figure 19: Top layer placement examples for BMI2xy/BMI3xy … 17  
Figure 20: Top layer placement examples for BHI3xy … 17  
Figure 21: Middle and bottom layer placement examples … 18  
Figure 22: Recommendation to keep the side of LGA free from solder material … 19  
Figure 23: Recommendation not to use underfill for LGA packages … 19

## 1 Purpose of this document

This document describes the recommended conditions and parameters to be applied when handling, soldering, and mounting Bosch Sensortec's Inertial Measurement Units (IMUs, incl. programmable IMUs) to a PCB. This document applies to all Sales Part Numbers mentioned on the cover sheet. In case the Sales Part Number of your Bosch Sensortec device is not listed, contact your Bosch Sensortec representative.

Important:

- To avoid damage to the sensor and the loss of warranty, strictly follow the instructions described in this document.
- It is strongly recommended to study the sensor datasheet before handling the sensor device.
- In case you have any other questions, do not hesitate to contact your Bosch Sensortec representative.

## 2 Package outline

Refer to the latest version of the corresponding product datasheet or preliminary datasheet.

## 3 Landing pattern

Refer to the latest version of the corresponding product datasheet or preliminary datasheet.

## 4 Moisture sensitivity level (MSL)

The moisture sensitivity level of the device corresponds to JEDEC Level 1. Also, see the following standards:

- IPC/JEDEC J-STD-020E "Joint Industry Standard: Moisture/Reflow Sensitivity Classification for non-hermetic Solid State Surface Mount Devices."
- IPC/JEDEC J-STD-033D "Joint Industry Standard: Handling, Packing, Shipping and Use of Moisture/Reflow Sensitive Surface Mount Devices."

Both standards are available on JEDEC's website.

The sensor fulfills the lead-free soldering requirements of the IPC/JEDEC standard mentioned above, i.e., reflow soldering with a peak temperature Tp up to 260°C.

## 5 RoHS compliance/halogen content

As listed on the cover sheet, the devices (Sales Part Numbers) comply with directive 2011/65/EU (RoHS) and its subsequent amendments, restricting the use of hazardous substances in electrical and electronic equipment.

The devices comply with the industry standard IEC 61249 for halogen content. For more details on the corresponding analysis results, contact your Bosch Sensortec representative.

Corresponding chemical analysis certificates are available as separate documents from Bosch Sensortec.

## 6 Mounting recommendations

MEMS sensors, in general, are high-precision measurement devices with electronic and mechanical silicon structures. Bosch Sensortec MEMS sensor devices are designed for precision, efficiency, and mechanical robustness.

To achieve the best possible results for your design, the following recommendations should be considered when mounting the sensor on a printed circuit board (PCB).

The scenarios described below, for example, may lead to a bending of the PCB, which consequently might influence the performance of the sensor mounted to the PCB. Due to the natural presence of eigenmodes in MEMS gyroscopes, straininduced changes in the modal landscape can, in unfavourable cases, also lead to offset anomalies.

It is recommended to use additional tools during the design-in phase to evaluate and optimize the considered placement position of the sensor on the PCB, for example:

-   
  infrared camera regarding thermal aspects
-   
  warpage measurements and/or FEM simulations regarding mechanical stress
-   
  drop tests of the device after soldering on the target application PCB regarding shock robustness

It is recommended to keep a reasonable distance between the sensor mounting location on the PCB and the critical locations described in the following examples. The exact value for a "reasonable distance" depends on the individual design and, therefore, must be determined case by case.

Contact your Bosch Sensortec representative if you have questions regarding the mounting of the sensor on your PCB or about evaluating and/or optimizing the considered placement position of the sensor on your PCB. If the recommendations mentioned above cannot be met, a specific in-line offset calibration after device placement onto your PCB might help minimize the potential remaining effects.

### 6.1 External loads/forces

In certain applications, circuit boards may feature an array of functional buttons or connection points. External loads or forces applied to these points can transmit stress through the board to the MEMS sensor, potentially causing mechanical damage. It is advisable to position MEMS sensors at a distance from these force-prone areas. See Figure 1.

![Figure 1: External loads/forces](images/p006-diagram-01.png)

> **Image description:** Block diagram of external loads/forces mounting recommendations.
>
> Components:
> force point/area
> Keep distance !
> sensor
> PCB
> base plate (e.g., shelf)
> anchor point (e.g., screw)
>
> Connections:
> The force point/area is shown applying a load to the base plate (e.g., shelf).
> The sensor is positioned relative to the base plate (e.g., shelf) and the PCB.
> The PCB is mounted on the base plate (e.g., shelf) and secured by the anchor point (e.g., screw).
> The Keep distance ! indicates a separation between the force point/area and the sensor.
>
> Values:
> No voltage, frequency, part value, or other number with a unit is shown.

*Figure 1: External loads/forces*

### 6.2 Thermal hot spots on the PCB

It is recommended to keep a reasonable distance from any thermal hot spots or areas with large thermal gradients, especially those that fluctuate over time when placing the sensor device to achieve higher accuracy measurements. See Figure 2. Hot spots can include a µController or a graphic chip, other integrated circuits with high power consumption, processors, batteries, power management circuitry, or high-current devices. Thermal gradients refer to the uneven variation of temperature across space or time. These temperature differences are typically caused by factors such as the power consumption of various components on the PCB, heat dissipation efficiency, and the material properties of the PCB.

The hot spots and thermal gradients on the PCB will have two effects: temperature rise will affect the performance of IMU series sensors, including the impact on the sensitivity and zero-g offset of the acceleration and gyroscope. Additionally, the rise in temperature will cause the PCB to deform, generating additional mechanical stress. Time-varying thermal gradients can cause uneven thermal expansion of PCB materials, resulting in dynamic warping that may affect measurement accuracy and introduce calibration errors. Choosing PCB materials (such as prepreg and laminate) with a smaller coefficient of thermal expansion (CTE) can reduce the strain caused by temperature variations.

![Figure 2: Thermal hot spots on the PCB](images/p007-diagram-01.png)

> **Image description:** Block diagram of mounting recommendations for thermal hot spots on the PCB.
>
> Components:
> Thermal hot spot (e.g., CPU)
> sensor
> PCB
> base plate (e.g., shelf)
> Thermal stable area
> anchor point (e.g., screw)
>
> Connections:
> The Thermal hot spot (e.g., CPU) is located on the PCB.
> The sensor is positioned relative to the PCB, with a dashed line indicating a "Keep distance !" between the Thermal hot spot (e.g., CPU) and the sensor.
> The PCB is mounted on the base plate (e.g., shelf).
> The anchor point (e.g., screw) is shown securing the PCB to the base plate (e.g., shelf).
> The Thermal stable area is indicated on the PCB, adjacent to the Thermal hot spot (e.g., CPU).
>
> Numbers:
> No voltage, frequency, part value, or other numbers with units are shown.

*Figure 2: Thermal hot spots on the PCB*

Prevent any contact between heat sinks or other external structures and the sensor, which may also create thermal conduction and mechanical stress. See Figure 3.

![Figure 3: Heat sinks](images/p007-diagram-02.png)

> **Image description:** Block diagram of mounting recommendations for thermal hot spots on the PCB.
>
> Components:
> heat sink
> PCB
> No !
> base plate (e.g., shelf)
> sensor
> anchor point (e.g., screw)
>
> Connections:
> The heat sink is positioned above the PCB.
> The sensor is positioned on the PCB, underneath the heat sink.
> The base plate (e.g., shelf) is below the PCB.
> The anchor point (e.g., screw) is shown on the sides of the PCB, connecting it to the base plate (e.g., shelf).
> There is a red 'No !' symbol indicating no contact between the heat sink and the sensor.
>
> Values:
> No voltage, frequency, part value, or other number with a unit is shown.

*Figure 3: Heat sinks*

### 6.3 Mechanical stress maximum on the PCB

It is recommended to keep a reasonable distance from any mechanical stress maximum when placing the sensor device.

Figure 4 shows a stress maximum in the center of the diagonal crossover of the four anchor points. It is good manufacturing practice to always avoid or reduce mechanical stress by optimizing the PCB design first, then placing the sensor in an appropriate low-stress area.

A best practice is to place MEMS components on the PCB in locations where the stress value is not exceeding 500 µstrain.

![Figure 4: Mechanical stress maximum on the PCB](images/p008-diagram-01.png)

> **Image description:** Block diagram of mechanical stress maximum on the PCB.
>
> Components:
> PCB
> sensor
> anchor point (e.g., screw)
>
> Connections:
> The PCB contains four anchor points (e.g., screw) located at the corners.
> The PCB contains a sensor located near the center where the diagonal lines cross.
> The diagram shows two states: one where the sensor is placed in a high-stress area (left image) and one where the sensor is placed in a low-stress area (right image).
> In the right image, there are dashed boxes around the sensor area with checkmarks, and an arrow points from the sensor area to the text "Keep distance !".
>
> Values:
> No voltage, frequency, part value, or other number with a unit is shown.

*Figure 4: Mechanical stress maximum on the PCB*

### 6.4 Distance to PCB anchor points

When placing the sensor device, keep a reasonable distance from anchor points where the PCB is fixed at a base plate (e.g., a shelf or something similar). See Figure 5.

![Figure 5: Distance to PCB anchor points](images/p008-diagram-02.png)

> **Image description:** Block diagram of mounting recommendations for a sensor device.
>
> Components:
> base plate (e.g., shelf)
> anchor point (e.g., screw)
> sensor
> PCB
>
> Connections:
> The sensor is positioned relative to the PCB, which is mounted on the base plate via anchor points. A dashed line indicates a "Keep distance !" between the sensor and the anchor point.
>
> Numbers:
> No voltage, frequency, part value, or other number with a unit is shown.

*Figure 5: Distance to PCB anchor points*

### 6.5 Vibrations on the PCB

Various measures can be applied to minimize vibration transmission, such as adding damping washers or foam pads at anchor points or optimizing the placement of the IMU to reduce the vibration effects.

It is recommended to position the IMU away from other sub-systems/modules (from the top and bottom side) on the PCB to mitigate or prevent the impact of vibrations from these parts to the sensor. MEMS gyroscopes and accelerometers have multiple internal eigenmodes. In the case of a resonant match between an external aggressor and an eigenmode of the sensor, even relatively low vibration amplitudes might cause damage to the MEMS structures. The system-level design should avoid vibrations that align with the resonant frequencies of the IMU.

External vibrations that coincide with the sensor's eigenfrequencies may introduce extra noise, extra offset, or even damage the sensor. Do not place the sensor next to components or in areas where the PCB's resonant amplitudes (vibrations) are likely to occur or to be expected.

It is recommended to contact your Bosch Sensortec representative to get the specific application notes in case vibration sources have to be placed on the same PCB. For the new-generation BMI series sensors, Bosch Sensortec will provide details on safe external aggressor loads to help make optimized platform integration easier.

The predominant vibration sources at the PCB board level are those listed below. Measures must be taken to avoid their impact on the sensor.

#### 6.5.1 Mechanical vibration sources

Vibrating components, such as speakers, vibration/haptic motors, fans, etc., should be mechanically isolated from the IMU.

#### 6.5.2 Electrical sources

A switched-mode power supply (SMPS) converts voltage through high-frequency switching, which can induce vibrations in its circuit components, such as inductors and capacitors. If the SMPS's specific switching frequencies (including harmonic frequencies) coincide with the sensor's specific eigen frequencies (including harmonic frequencies), it could result in increased noise and offset in the sensor gyroscope or even cause damage to the MEMS.

#### 6.5.3 MLCCs

Multilayer ceramic chip capacitors (MLCCs) are commonly employed in wireless and fast-charging ICs and DC-DC conversion circuits. Some ceramic capacitors may exhibit vibration or low-frequency buzzing with very high acceleration loads, known as "singing capacitors," due to imbalances in the power circuit. This phenomenon, attributed to the piezoelectric effect, can induce vibrations overlapping with the resonant frequency of the IMU, thereby impacting sensor performance or causing damage to the fine structures inside the MEMS. Furthermore, prepolarization from DC bias enables AC voltages to generate mechanical stresses, resulting in the deformation of the MLCC and the attached PCB substrate. See Figure 6.

![Figure 6: Deformation of MLCC affected by electric field](images/p009-diagram-01.png)

> **Image description:** Figure 6: Deformation of MLCC affected by electric field. This is a top view of a rectangular package or module. Pin 1 is not explicitly indicated.
>
> The package has multiple pins visible around its perimeter. The pins are not explicitly numbered or named in the drawing.
>
> No dimensions with units and tolerances are shown in the figure.
>
> Text inside the figure reads: "Direction of applied electric field".

*Figure 6: Deformation of MLCC affected by electric field*

**Recommendations for PCB material and thickness:**

- Increasing PCB thickness resists deformation and reduces (body) sound pressure level (SPL).

**Optimize the layout of MLCCs on the PCB:**

- MLCCs are placed as far away from the sensor as possible.
- Prioritizing placing MLCCs on PCB edges produces a lower SPL.
- When MLCCs are placed symmetrically on opposite sides of a PCB, the vibrations generated by each other can often be canceled out.
- When placed in an L-shape or T-shape layout configuration, the vibrations of the two MLCCs are orthogonal to each other and provide a certain level of cancellation. See Figure 7.

![Figure 7: Placement of MLCCs to significantly reduce vibration](images/p010-diagram-01.png)

> **Image description:** Block diagram of Figure 7: Placement of MLCCs to significantly reduce vibration.
>
> Components:
> Two MLCCs
> PCB
> L-Shape
> T-Shape
>
> Connections:
> Two MLCCs are placed on opposite sides of the PCB.
> The L-Shape and T-Shape show the placement of two MLCCs with the same electrical signal.
>
> Numbers and values:
> No voltage, frequency, part value, or other number with a unit is shown.

*Figure 7: Placement of MLCCs to significantly reduce vibration*

- MLCCs can be replaced with tantalum capacitors; the structural composition of tantalum capacitors prevents them from generating vibration and deformation effects similar to those of MLCCs on the PCB board.

It is recommended that the impact of MLCCs on IMUs can be reduced or avoided in the early design-in phase from the following aspects:

- Consider aggressor frequencies (e.g., frequency of fast charging and/or wireless charging ICs) and ideally test more than one fast-charging IC frequency.
- Invest in Laser Doppler Vibrometer (LDV) measurements to scan the vibration profile of the target PCB and assess expected load (g's) at available placement position (s) of IMU.
- Perform finite element modeling (FEM) simulations to understand the impact of aggressors better.
- Qualify the IMU based on HSMI recommendations.

By implementing best practices, vibrations caused by MLCCs can be reduced but not entirely prevented. If switching the frequency hits an eigenmode of the IMU, vibration reduction alone cannot guarantee the absence of interference. If you have any questions about the placement of MLCCs, contact your Bosch Sensortec representative.

#### 6.5.4 PCB with large span

Avoid placing the IMU on the end of a long hanging beam or the PCB with a large span. See Figure 8.

![Diagram on page 11](images/p011-diagram-01.png)

> **Image description:** Block diagram of mounting recommendations for a long hanging beam or large span PCB.
>
> Components:
> - base plate (e.g., shelf)
> - anchor point (e.g., screw)
> - sensor
> - PCB with large span (e.g., > 30mm)
>
> Connections:
> - The base plate connects to the anchor point.
> - The anchor point connects to the PCB with large span.
> - The sensor is positioned above the PCB with large span, with a dashed line indicating a required distance between the sensor and the PCB.
>
> Values:
> - PCB with large span has a minimum span of > 30mm.

### 6.6 Resin coatings and shields

#### 6.6.1 Encapsulation/over-molding/resin coatings

As shown in Figure 9 and Figure 10, avoid encapsulating/over-molding or even partially covering the sensor with any protective materials, such as epoxy resin. Doing so can lead to an asymmetric stress distribution over the sensor package.

![Figure 9: Encapsulation/over-molding/resin coatings fully cover the sensor](images/p011-diagram-02.png)

> **Image description:** Block diagram of mounting recommendations regarding encapsulation.
>
> Components:
> Encapsulation/over-molding/resin coating
> PCB
> base plate (e.g., shelf)
> sensor
> anchor point (e.g., screw)
>
> Connections:
> The Encapsulation/over-molding/resin coating is shown covering the sensor, and a red 'X' with "No !" is placed over this coverage.
> The sensor is positioned above the base plate (e.g., shelf).
> The PCB is shown adjacent to the sensor and base plate.
> The anchor point (e.g., screw) is shown on the base plate.
>
> Values:
> No voltage, frequency, part value, or other number with a unit is shown.

*Figure 9: Encapsulation/over-molding/resin coatings fully cover the sensor*

![Picture on page 12](images/p012-picture-02.png)

> **Image description:** Rendering of a cross-section illustrating mounting recommendations.
>
> Visible markings include:
> protected device (e.g., CPU)
> resin coating
> sensor
> PCB
> No !
> base plate (e.g., shelf)
> anchor point (e.g., screw)
>
> No dimensions or scale are shown.

![Figure 10: Resin coatings partially cover the sensor](images/p012-diagram-01.png)

> **Image description:** Block diagram of mounting recommendations illustrating the placement of a resin coating over a sensor.
>
> Components:
> protected device (e.g., CPU)
> resin coating
> sensor
> PCB
> No !
> base plate (e.g., shelf)
> anchor point (e.g., screw)
>
> Connections:
> The protected device (e.g., CPU) is shown resting on the base plate (e.g., shelf).
> The resin coating is shown covering the protected device (e.g., CPU) and partially covering the sensor.
> The sensor is shown mounted on the PCB.
> The PCB is shown resting on the base plate (e.g., shelf).
> The anchor point (e.g., screw) is shown securing the base plate (e.g., shelf) to an unlabelled lower structure.
> The "No !" indicator is shown with red lines crossing over the sensor, indicating an incorrect placement or condition.
>
> Values:
> No voltage, frequency, part value, or other number with units is shown.

*Figure 10: Resin coatings partially cover the sensor*

#### 6.6.2 Shields

A shield can cause mechanical stress on the PCB, which may have the effect of causing additional offset of sensors located close to the shield. Keep the sensor away from the shield, including the opposite side of the shield. See Figure 11.

![Figure 11: Shields](images/p012-diagram-03.png)

> **Image description:** Block diagram of mounting recommendations for shields.
>
> Components:
> shield
> sensor
> PCB
> base plate (e.g., shelf)
> shielded device
> anchor point (e.g., screw)
>
> Connections:
> The shield is positioned above the base plate (e.g., shelf) and is connected to the PCB.
> The shielded device is positioned above the base plate (e.g., shelf) and is connected to the PCB.
> The sensor is positioned above the base plate (e.g., shelf) and is connected to the PCB.
> The base plate (e.g., shelf) is supported by anchor point (e.g., screw) on both the left and right sides.
> There is a dashed line indicating a required distance between the shield and the sensor, labeled "Keep distance!".
>
> Values:
> No voltage, frequency, part value, or other number with units is shown.

*Figure 11: Shields*

### 6.7 PCB precautions

- Keep the IMU at least 2 mm away from PCB edges and bridges or V-Grooves (see Figure 12) to ensure optimal performance, as deflection caused by milling drills or saws when separating the PCB can damage the MEMS device.
- Avoid dull milling cutters and saw blades, which can cause excessive mechanical vibration.
- Do not break the panel. Severe bending force and mechanical shock may damage the IMU.

![Figure 12: Edge of PCB](images/p013-diagram-01.png)

> **Image description:** Block diagram of PCB mounting precautions.
>
> Blocks:
> main PCB
> array PCB
> V-Groove
> bridge
> sensor
>
> Connections:
> An arrow points from main PCB to array PCB.
> An arrow points from main PCB to a location labeled bridge.
> An arrow points from main PCB to a location labeled V-Groove on the right.
> An arrow points from main PCB to a location labeled V-Groove on the bottom right.
> An arrow points from array PCB to a location labeled bridge.
> An arrow points from array PCB to a location labeled sensor.
> An arrow points from sensor to a location labeled Keep distance !
> An arrow points from array PCB to a location labeled V-Groove on the left.
> An arrow points from array PCB to a location labeled bridge.
>
> Numbers:
> No voltage, frequency, part value, or other number with a unit is shown.

*Figure 12: Edge of PCB*

### 6.8 Multiple IMUs on the same PCB

When multiple IMUs are mounted on the same PCB with close gyroscope drive frequencies (delta of drive frequencies inside the output signal bandwidth), mechanical resonance or frequency coupling may occur. Oscillation energy can transfer through the PCB, leading to tones in the rate signal and offset change.

It is strongly recommended not to place two or more IMUs on the same PCB.

In cases where it is necessary to use multiple IMUs on the same PCB, at least one of the following strategies should be implemented:

- Mechanically isolate IMUs from each other to break the vibration transmission path. Cutting PCB slots is a recommended method, as illustrated in Figure 13.

![Figure 13: PCB cut slot for multiple IMUs on the same PCB](images/p013-diagram-02.png)

> **Image description:** Block diagram of PCB cut slot for multiple IMUs on the same PCB.
>
> Components:
> IMU (two instances)
> PCB cut slot
> Single PCB
>
> Connections:
> One IMU connects to the Single PCB.
> The other IMU connects to the PCB cut slot.
> The PCB cut slot connects to the Single PCB.
>
> Numbers, voltages, frequencies, or part values:
> None are shown.

*Figure 13: PCB cut slot for multiple IMUs on the same PCB*

- Avoid running multiple IMUs simultaneously and enable only one gyroscope at a time.

Note: Avoid using dip, spray, or partial coating methods to alter the mechanical connection between the sensor and PCB, as this could potentially affect sensor performance and board-level reliability.

### 6.9 Magnetic, Electric, Infrared (IR), and Radio Frequency (RF)

Avoid mounting (and operating) the sensor in the vicinity of strong magnetic, electric, infrared (IR), and radio frequency (RF) radiation fields.

### 6.10 Electrostatic charging

Avoid electrostatic charging of the sensor and device wherein the sensor is mounted.

## 7 PCB Design Guidelines

- For the solder mask design of each pin, it is recommended to use Non-Solder Mask Defined (NSMD) pads. NSMD and SMD (Solder Mask Defined) are illustrated in Figure 14.

![Figure 14: NSMD and SMD](images/p014-diagram-01.png)

> **Image description:** Figure type: pinout or package drawing
> Section: 7 PCB Design Guidelines
> Caption: Figure 14: NSMD and SMD
>
> The image displays two representations, NSMD and SMD, illustrating PCB design guidelines related to solder mask and PCB land.
>
> NSMD representation:
> This is a top view of the NSMD package. Pin 1 is not explicitly indicated.
> The components shown are:
> solder mask openings
> solder mask
> overlay
> PCB land
>
> SMD representation:
> This is a top view of the SMD package. Pin 1 is not explicitly indicated.
> The components shown are:
> solder mask openings
> solder mask
> overlay
> PCB land
>
> No dimensions with units and tolerances are shown in the figure.

*Figure 14: NSMD and SMD*

- Placing any structure or routing on the top layer of the PCB below the sensor is strictly prohibited.
- The symmetry of the traces connected to the pads can optimize component self-alignment and promote improved solder paste reduction after reflowing, so it is recommended to maintain symmetry and balance of traces on the pads as much as possible.
- Keep the distance between the screw hole and the sensor greater than 2 mm for best performance.
- Confirm that the pin #1 indicator is unconnected and must remain unconnected to avoid affecting sensor functionality.

### 7.1 Footprint design rules

All lands are required to be of equal size and do not need to be large. General recommendations for PCB pads and solder mask layers are illustrated in Figure 15. Refer to the datasheet for specific pad count, size, and pitch.

![Figure 15: Footprint design rules](images/p015-diagram-01.png)

> **Image description:** Pinout or package drawing. Top view. Pin 1 is not shown.
>
> Pins are not numbered or named in the drawing.
>
> Dimensions:
> D: unreadable
> A: unreadable
> C: unreadable
> B: unreadable
>
> Text elements:
> solder mask opening placed outside of land footprint
> package footprint
> PCB land
> D
> A
> C
> B

*Figure 15: Footprint design rules*

PCB land and connecting traces should be designed symmetrically.

- A = PCB land length = LGA solder pad length B = PCB land width = LGA solder pad width C = Solder mask opening length (longer than A) D = Solder mask opening width (wider than B)

### 7.2 Stencil design

The stencil design is very important for sensor soldering. See Figure 16.

- It is recommended to use a stainless-steel stencil to apply solder paste.
- Stencil thickness will result from pad size, aspect, and area Ratio.
- The stencil opening dimensions need to follow the general best practices for stencil design.
- Aspect ratio of stencil openings: Aperture width/stencil thickness = W/T > 1.5
- Area ratio of stencil openings: Aperture area/aperture wall cross-sectional area = (W x L)/ [2 x (W+L) x T] > 0.66

![Diagram on page 15](images/p015-diagram-03.png)

> **Image description:** Package drawing of a rectangular component viewed from the top. Pin 1 is not indicated.
>
> No pins are labeled.
>
> Dimensions shown are:
> W: unreadable
> T: unreadable
> L: unreadable

![Figure 16: Stencil opening dimensions](images/p015-picture-02.png)

> **Image description:** Figure 16: Stencil opening dimensions. This is a top view of a stencil opening on a PCB. Pin 1 is not indicated.
>
> There are no numbered pins or pin names shown.
>
> The dimensions shown are:
> W: unreadable
> T: unreadable
> L: unreadable

*Figure 16: Stencil opening dimensions*

### 7.3 PCB design rules for the traces

All traces must flow outside the component, remain symmetrical, and be parallel to the long sides of the pads.

The traces must remain the same thickness; power signals do not require thicker traces because the current flowing into them is very low, thus avoiding potential mechanical stress.

It is not recommended the ground plane be connected directly to the package pad. It is better connected via standard traces. See Figure 17.

![Picture on page 16](images/p016-picture-02.png)

> **Image description:** Rendering of a PCB layout from the top side.
>
> Visible markings include:
> - C1
> - C2
> - U1
>
> The layout shows traces connecting components C1, C2, and U1. U1 is a square integrated circuit package with visible pads and connections. Traces are colored red, blue, and yellow. There are several circular pads visible around the components.
>
> No dimensions or scale are shown.

![Figure 17: Correct traces](images/p016-diagram-01.png)

> **Image description:** Schematic of correct traces for PCB design.
>
> Components:
> U1
> C1
> C2
>
> Connections:
> C1 connects to a red trace on the left.
> C2 connects to a red trace on the left.
> U1 connects to several traces:
> - One connection goes to a red trace on the left.
> - Several connections go to blue traces on the right.
> - One connection goes to a red trace below it.
> - One connection goes to a red trace above it.
>
> Numbers and values:
> No voltage, frequency, part value, or other numbers with units are shown.

*Figure 17: Correct traces*

High-frequency communication lines (as SCx or OSCx) should be routed with enough distance of several millimeters from the sensor. The traces to the clock pads should extend in a direction that is as perpendicular as possible to the closest edge of the sensor package to reduce coupling into sensitive areas.

When designing the layout, as much as possible, ensure that VDD, VDDIO, and serial interface traces are isolated from any components and signals related to battery charging and DC-DC switching power regulation. The substantial energy carried by power signals can induce significant noise or spikes.

If the above requirements cannot be met during actual wiring, the 3W principle can be followed, where the center spacing of high-speed signal traces is not less than three times the line width. See Figure 18.

![Figure 18: High-speed traces](images/p016-diagram-03.png)

> **Image description:** Block diagram of PCB design rules for high-speed traces.
>
> Blocks:
> High speed traces
> GND
> IMU Power/Serial interface traces
> W
>
> Connections:
> A green double-headed arrow labeled "keep distance" connects the High speed traces and GND.
> A green dashed arrow connects the High speed traces to the 3W principle section.
> The 3W principle section shows a blue block connected to a gray block via green arrows labeled "W" and "W".
> A green double-headed arrow labeled "W" connects the blue block and the gray block in the 3W principle section.
>
> Numbers and values:
> The text states "signal traces is not less than three times the line width."
> The text states "See Figure 10."
> The text states "See Figure 18."
> The text states "3W principle".

*Figure 18: High-speed traces*

### 7.4 BMI2xy, BMI3xy and BMI4xy placement rules for the top layer

For all MEMS devices, soldering takes place exclusively on the top layer. Avoid positioning any routing or vias directly under the device on the top side. See Figure 19.

![Figure 19: Top layer placement examples for BMI2xy/BMI3xy](images/p017-picture-01.png)

> **Image description:** Rendering of three PCB layout examples from the top layer.
>
> Visible markings include:
> U1
> C2
> WRONG: Routing under the sensor on TOP layer
> WRONG: Via under the sensor on TOP layer
> RIGHT: leave free space under the sensor on TOP layer
>
> No dimensions or scale are shown.

*Figure 19: Top layer placement examples for BMI2xy/BMI3xy*

### 7.5 BHI3xy placement rules for the top layer

BHI3xy is pin-to-pin compatible with BMI2xy and BMI3xy for outer pads. In addition, BHI3xy has six inner pads.

- For inner pads, blind vias can be used to route the signals.
- The size of the blind vias should not exceed the pad size.
- The blind vias must comply with a drilling aspect ratio of 1:1 or greater, where the aspect ratio is defined as the hole size divided by the via-through layer thickness. See Figure 20.
- Note that other top-layer placement rules are consistent with the BMI2xy and BMI3xy.

![Figure 20: Top layer placement examples for BHI3xy](images/p017-picture-02.png)

> **Image description:** Rendering of three PCB layout examples for the top layer.
>
> Left image: A close-up view of a red PCB surface showing traces and pads. Visible markings include "S2" and "S3". The caption below reads: "RIGHT: Place blind vias within central pads for signal routing".
>
> Center image: A 3D rendering of a cylindrical component mounted on a green PCB. Visible labels include "hole size" with horizontal dimension lines, "via-through layer thickness" with a vertical dimension line, and the caption below reads: "RIGHT: Drilling aspect ratio of 1:1 or greater".
>
> Right image: A close-up view of a red PCB surface showing traces and pads. Visible markings include "S2" and "S3". A yellow dotted line highlights a via placement. The caption below reads: "WRONG: Via under the sensor except for the central pads".
>
> The figure is titled "Figure 20: Top layer placement examples for BHI3xy" and is from Section 7 PCB Design Guidelines > 7.5 BHI3xy placement rules for the top layer. The accompanying text mentions that BHI3xy is pin-to-pin compatible with BMI2xy and BMI3xy for outer pads, and BHI3xy has six inner pads.

*Figure 20: Top layer placement examples for BHI3xy*

**Note:**

- To ensure full compatibility with other BMI2xy and BMI3xy sensors, additional points outlined in Chapter 4.2 of BHI3xy datasheet also need to be considered.
- If the customer solders the BMI2xy using the BHI3xy landing pattern for a compatible design to prevent a decrease in BM2xy performance, it is recommended the customer prepare a separate set of stencils for BMI2xy soldering.

### 7.6 Placement rules for the middle and bottom layers

Power plane or signal routing can be done on the middle and the bottom layer below the sensor. See Figure 21. Note that this bottom placement rule only applies to MEMS accelerometers, gyroscopes, and pressure sensors. The BMI2xy/BMI3xy and BHI3xy share these rules.

![Figure 21: Middle and bottom layer placement examples](images/p018-picture-01.png)

> **Image description:** Rendering of two PCB layer examples from the middle and bottom layers under a sensor.
>
> Left image:
> - Component labels visible: C1, C2, U1.
> - Text label: RIGHT: Power plane on MIDDLE or BOTTOM layer under the sensor.
> - The image displays a power plane routing pattern on the PCB layer.
>
> Right image:
> - Component labels visible: C1, C2, U1.
> - Text label: RIGHT: Signal routing on MIDDLE or BOTTOM layer under the sensor.
> - The image displays a signal routing pattern on the PCB layer.
>
> No dimensions or scale are shown.

*Figure 21: Middle and bottom layer placement examples*

## 8 Note on internal package structures

Within the scope of Bosch Sensortec's ambition to improve its products and secure the product supply while in mass production, Bosch Sensortec qualifies additional sources for the LGA package of its sensors.

While Bosch Sensortec took care that all of the technical package parameters as described above are 100% identical for both sources, there can be differences in the chemical analysis and internal structure between the different package sources.

However, as secured by the extensive product qualification processes at Bosch Sensortec, this has no impact on the usage or the quality of the sensor.

## 9 Device marking

Refer to the latest version of the corresponding product datasheet or preliminary datasheet.

## 10 Pick and place guidelines

The IMU product is a surface mount device (SMD) that requires surface mount technology (SMT) for mounting onto a PCB using high-speed, high-precision automated equipment. SMT machines typically use nozzles to pick components from a predetermined area and accurately place them at specified soldering locations. This process must be strictly controlled, as the nozzle position, pick and place force, and mechanical parameters involved may damage the structure of the MEMS sensor being handled.

To minimize the risk of damage to MEMS sensors during the SMT process, consider the following key factors:

- Ensure the equipment can trigger an alarm to alert operators in case of mispicking or uncontrolled pickup height.
- Avoid any improper contact, impact, or pressure from mechanical equipment or components on the IMU or its PCB.
- Ink-jet printing, laser marking, or any form of direct marking on the IMU is not recommended as it may induce damage to the MEMS element or compromise sensor performance by thermal or mechanical stress.
- Remove unnecessary hazardous tools, such as screwdrivers and scissors, to prevent accidental damage to the PCB and components.
- Correctly set the nozzle vacuum pressure level to avoid excessive force that could damage the MEMS structure.

- Use pick and place machines with Z-axis dynamic control technology, such as high-precision pressure feedback control, to ensure stable component placement and prevent impact or excessive force on the MEMS structure.

## 11 Reflow soldering

### 11.1 Recommendation for soldering of sensors in the LGA package

Ensure that the edges of the LGA substrate of the sensor are free of solder material. It is not recommended to allow solder material to form a high meniscus covering on the edge of the LGA substrate (see Figure 22) since metal traces of the internal LGA substrate wiring are exposed.

![Figure 22: Recommendation to keep the side of LGA free from solder material](images/p019-diagram-01.png)

> **Image description:** Figure type: pinout or package drawing
> Section: 11 Reflow soldering > 11.1 Recommendation for soldering of sensors in the LGA package
> Caption: Figure 22: Recommendation to keep the side of LGA free from solder material
>
> The image displays a cross-sectional view illustrating a recommendation for soldering a sensor package onto a PCB.
>
> Components shown from top to bottom are:
> sensor mold
> BT substrate of sensor
> PCB
>
> Text annotations within the figure include:
> Recommend: solder not covering substrate edge
> Not recommend: solder meniscus covering substrate edge
>
> The figure illustrates a cross-section showing the sensor mold, the BT substrate of the sensor, and the PCB. It highlights a recommendation that solder should not cover the substrate edge, and advises against a solder meniscus covering the substrate edge.

*Figure 22: Recommendation to keep the side of LGA free from solder material*

Using underfill (e.g., copper underfill) for the LGA package is forbidden. See Figure 23.

![Figure 23: Recommendation not to use underfill for LGA packages](images/p019-diagram-02.png)

> **Image description:** Figure type: pinout or package drawing
> Section: 11 Reflow soldering > 11.1 Recommendation for soldering of sensors in the LGA package
> Caption: Figure 23: Recommendation not to use underfill for LGA packages
>
> The image shows a cross-sectional view of a sensor package assembly. Pin 1 is not explicitly indicated.
>
> The components shown are:
> sensor mold
> BT substrate of sensor
> solder pad
> solder material
> PCB
>
> Text within the figure indicates:
> Allowed: no underfill
> Forbidden: usage of underfill(red)
>
> No dimensions with units and tolerances are shown.

*Figure 23: Recommendation not to use underfill for LGA packages*

### 11.2 Classification reflow profiles

The sensor fulfills the JEDEC and lead-free soldering requirements described in Chapter 4, i.e., reflow soldering (after MSL1 pretreatment) with a peak temperature Tp up to 260 °C.

### 11.3 Multiple reflow soldering cycles

The product can withstand up to 3 reflow soldering cycles in total. In case 5 cycles are required, contact your local Bosch Sensortec representative.

## 12 Desoldering recommendations

The desoldering process for MEMS products is critical. Follow the recommendations below to prevent device damage and ensure the integrity and performance of the components.

### Temperature Control

- Use a controlled heat source (220 °C - 250 °C) and avoid exceeding 260 °C (maximum peak temperature).
- Limit heating time to 30 seconds to prevent internal stress and pad detachment.
- The desoldering tool maintains a stable and controlled temperature, preferably monitored through temperature sensor feedback, to prevent overheating and potential device damage.

### Minimize Mechanical Stress

- Ensure that the solder is fully melted before the sensor is removed.
- Avoid prying or applying excessive force to prevent pad or MEMS structure damage.

### ESD (Electrostatic Discharge) Protection

- Handle the sensor in an ESD-safe environment with grounding measures.
- Avoid direct contact with solder pads to prevent static discharge.

### Moisture Sensitivity

- Follow MSL (Moisture Sensitivity Level) storage guidelines, see Chapter 4.
- If the sensor has been exposed to moisture, bake it before desoldering to prevent the popcorn effect.

Note: Do not resolder sensors that have been desoldered. Resoldering previously desoldered components may affect their reliability and performance.

## 13 Customer Return

In case a part needs to be returned to Bosch Sensortec for failure analysis (FA) purposes, it is preferred that the whole PCB with an assembled sensor is provided. Removing the PCB area where the sensor is located may cause severe damage to the internal MEMS structure and make failure analysis impossible. If this is not possible, consider carefully cutting a larger area around the part while avoiding the warpage of the assembly.

If neither of the aforementioned recommendations for re-using the PCB is possible, desolder the sensor following the recommendations in section 12. Use this method only if necessary and avoid additional manual handling. It needs to be noted that different IMU generations may require dedicated test sockets when measured in an unsoldered state, even if their pin-out and package dimensions are identical, due to differences of the internal MEMS element and contact sensitivity.

When shipping the part, carefully pack it to prevent ESD and mechanical damage during transportation. It is recommended to place the part in an ESD bag, wrap the bag in foam or bubble materials, and ship it in a stable package. These procedures are required to ensure that the failure analysis (FA) can be conducted at Bosch Sensortec successfully.

In addition, please get in contact with your local Bosch Sensortec representative for further information, support, and delivery of the desoldered parts.

**Note**: If any underfill/encapsulation/over-molding was used (not recommended, see section 6.6 and 11.1) and the part needs to be returned to Bosch Sensortec for FA, please provide details on the type of materials used for the underfill/encapsulation/over-molding (e.g., silicone-based) to facilitate chemical removal at Bosch Sensortec for FA. Also, in such cases, the FA might take longer, and the reliability of the FA results cannot be guaranteed.

## 14 Tape & reel

### 14.1 Tape & reel specification

![Diagram on page 21](images/p021-diagram-01.png)

> **Image description:** Package drawing, top view. Pin 1 is not explicitly indicated.
>
> Pins are labeled as Bo and A0, but no numbered pins are explicitly listed in order.
>
> Dimensions:
> 0.30 ±.05
> R.0.30 MAX.
> 2.00 ±.05 SEE NOTE 3
> 4.00 SEE NOTE 1
> 8.00
> Ø 1.50 MIN
> Ø 1.5 +0.1/-0.0
> 1.75 ±.10
> 12.0 +0.3/-0.1
> 5.50 ±.05 SEE NOTE 3
> A0
> Bo
> .12
> R.25

#### A0 = 3.30mm, B0 = 2.80mm, K0 = 1.10mm

**Note:**

1. Sprocket hole pitch cumulative tolerance ±0.1mm
2. Tolerances unless noted: ±0.1mm
3. Pocket position relative to sprocket hole measured as the true position of the pocket, not the pocket hole
4. Camber in compliance with EIA481
5. A0 and B0 are calculated on a plane at a distance "R" above the bottom of the pocket.

### 14.2 Orientation within the reel1

![Diagram on page 21](images/p021-diagram-02.png)

> **Image description:** Block diagram of a tape reel showing component orientation.
>
> Components:
> There are multiple identical blocks arranged in a row. Each block contains a square shape with a black center and a surrounding outline. Above the row of blocks, there is a label "Processing Direction" with an arrow pointing to the right.
>
> Connections:
> The blocks are arranged linearly, suggesting a sequence or flow in the "Processing Direction." There are no explicit signal lines or pin names connecting the blocks to each other, only their sequential arrangement.
>
> Numbers and Values:
> No voltage, frequency, part value, or other numerical values with units are shown in the diagram.

1not valid for SPNs 0273017083, 0273017095, see the respective datasheet.

## 15 Further important mounting, assembly & handling recommendations

Bosch Sensortec's IMUs are designed to sense acceleration and rate of rotation with high accuracy, even at low amplitudes, and contain highly sensitive structures inside the sensor element. The MEMS sensor can tolerate mechanical shocks up to several thousand g's. However, these limits might be exceeded in conditions with extreme shock loads, such as a hammer hitting the sensor or next to the sensor, dropping the sensor onto hard surfaces, etc.

We strongly recommend avoiding any g-forces beyond the limits specified in the datasheet during the sensors' transport, handling, and mounting in a defined and qualified installation process.

The IMUs have built-in protections against high electrostatic discharges or electric fields (2kV HBM); however, anti-static precautions should be taken for any other CMOS component.

Unless otherwise specified, proper operation can only occur when all terminal voltages are kept within the supply voltage range. Unused inputs must always be connected to a defined logic voltage level.

Ultrasonic cleaning and welding are strongly not recommended, as they may cause severe damage to the sensor's internal MEMS structures.

## 16 Document history and modification

| Rev. No | Chapter | Description of modification/changes | Date |
| --- | --- | --- | --- |
| 0.5 | all | Document creation | 3 Apr 2018 |
| 0.5.1 | Cover<br>9.2<br>10 | Updated Part#<br>Updated Table + Graphic<br>Spec, Graphic + Definition included | 18 Sep 2018 |
| 0.5.2 | all | Document name change to MIS-HS001 | 28 Nov 2018 |
| 1.0 | Cover<br>4<br>5<br>6.2.6<br>9.2<br>11 | Included Part# 0.273.017.003, -.004, -.008<br>Updated to latest JEDEC STD<br>Sentence rephrased<br>Updated content<br>Shortened content<br>Sentence rephrased<br>Updated Legal Disclaimer | 09 Oct 2019 |
| 1.1 | All<br>Cover<br>Cover<br>Cover<br>1<br>5<br>6.1<br>6.2.4<br>9.3<br>11 | Updated template<br>Legal Disclaimer replaced by reference page 1 "Notes"<br>Included BMI32x, Part# 0 273 017 018, 0 273 017 028<br>Excluded Part# 0 273 017 002, 0 273 017 003, 0 273 017 004 (EOL)<br>Sentence rephrased<br>Updated directive<br>Radio frequency environments + other components included<br>Updated content<br>5 cycles included<br>Sentence rephrased | 17 Feb 2022 |
| 1.2 | all<br>Cover | Updated template<br>Included Part# 0.273.017.037, -.039, added BHI3xx in title | 19 Dec 2022 |
| 1.3 | Cover | Included Part# 0.273.017.029 | 06 Oct 2023 |
| 1.4 | Cover<br>Cover<br>Cover<br>Cover<br>All<br>All<br>6<br>7 | Document title changed<br>Cover image changed<br>Included Part# 0 273 017 056<br>Excluded Part# 0 273 017 037, 0 273 017 039<br>Cover and header subtitles changed<br>Grammar modification<br>Content merged from Rev 1.3 and updated<br>Added content | 25 Apr 2024 |
| 1.5 | Cover<br>Cover<br>7 | Cover image changed<br>Included Part# 0 273 017 037, 0 273 017 039, added BHI3xy in title<br>Added BHI3xy layout rules | 14 Nov 2024 |
| 1.6 | Cover<br>6.2<br>6.5<br>6.6.1<br>6.8<br>10<br>11<br>12<br>13<br>15 | Included Part# 0 273 017 061, - 063, - 064, - 068<br>Updated content<br>Updated content<br>Added content<br>Added content<br>Added content<br>Updated content | 28 May 2025 |
| 1.7 | Cover | Included Part# 0 273 017 066, - 083, - 084, - 085, - 086, - 095 | 26 Aug 2026 |
|   | 14.1 | Update vector graph and note contents |   |
|   | 14.2 | Added footnote |   |

**Bosch Sensortec** | HSMI for BMI2xy/BMI3xy/BMI4xy/BHI3xy IMUs 25 | **25**

© Bosch Sensortec GmbH 2024, All rights reserved, also regarding any disposal, exploitation, Document number: BST-MIS-HS001-09 reproduction, editing, distribution, as well as in the event of applications for industrial property rights

**Bosch Sensortec GmbH** Gerhard-Kindler-Straße 9 72770 Reutlingen / Germany contact@bosch-sensortec.com www.bosch-sensortec.com

Modifications reserved

Specifications subject to change without notice Document number: BST-MIS-HS001-09
