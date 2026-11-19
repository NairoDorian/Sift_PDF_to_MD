## BST-BMM350-HS000-08

Document revision 1.8

Document release date April 2026  
Document number BST-BMM350-HS000-08  
Sales part number 0273.017.027  
Notes Data and descriptions in this document are subject to change without notice. Product photos and pictures are for illustration purposes only and may differ from the real product appearance.

# BMM350 Handling, Soldering and Mounting Instruction

![Picture on page 1](images/p001-picture-01.png)

> **Image description:** Rendering of an electronic component, viewed from the top side.
>
> Visible markings include:
> - BMM
> - 350
> - BOSCH
>
> No dimensions or scale are shown.

## Table of Contents

1 Purpose of this document … 4  
1.1 Who should read this manual? … 4  
2 Device Marking … 4  
3 PCB Design and Device Placing recommendations … 4  
3.1 Push-button contacts … 4  
3.2 Thermal hot spots on the PCB … 5  
3.3 Distance to PCB anchor points … 5  
3.4 Vibrations on the PCB … 5  
3.5 Redundant PCB anchor points … 6  
3.6 Mechanical stress maximum on the PCB … 6  
3.7 Resin coatings … 7  
3.8 RF electric field sources … 7  
3.9 PCB surface finish … 7  
4 PCB placement guideline based on magnetic influences … 7  
4.1 Non-geomagnetic field sources … 7  
On-board: Hard ferromagnetic materials … 7  
On-board: Current-induced fields … 8  
Comparison of geomagnetic field strength to other fields … 8  
4.2 Distortion and shielding by soft magnetic materials … 9  
On-board: Distortion … 9  
On-board and external: Shielding … 9  
Overview of external influences … 10  
4.3 Possibilities and limitations of software compensation … 10  
4.4 PCB placement advice … 12  
4.5 Hard magnetic offsets … 12  
PCB Currents … 12  
Soft magnetic effects … 14  
Overview … 14  
5 PCB layout recommendation … 15  
5.1 Dimensions and landing pattern recommendation … 15  
5.2 Connection diagram … 16

5.3 Layout recommendations … 16  
6 Reflow Soldering & Moisture Sensitivity Level … 17  
7 Wafer Level Chip Scale Package Handling … 17  
8 Handling of reels … 18  
8.1 Storage … 18  
8.2 Introduction into production … 18  
8.3 Orientation of sensors on the reel … 19  
8.4 Reel Dimensions … 20  
9 Returning WLCSP for Failure Analysis … 20  
10 Environmental Safety … 21  
10.1 RoHS … 21  
10.2 Halogen content … 21  
11 Legal Disclaimer … 22  
11.1 Engineering samples … 22  
11.2 Product use … 22  
11.3 Application examples and hints … 22  
12 Document history and modification … 23

## 1 Purpose of this document

This document describes the conditions and parameters to be applied when handling, soldering, and mounting the BMM350 to a PCB.

### 1.1 Who should read this manual?

This document is intended as a practical guidance for

- Hardware designer in terms of device handling, mounting, placement and environmental influences in the target system.
- Application engineers to deal with the disturbances caused due to the external magnetic fields.
- In addition, it will also explain which phenomena can and cannot be compensated for by software.

## 2 Device Marking

Please refer to the latest version of the corresponding product datasheet.

## 3 PCB Design and Device Placing recommendations

MEMS sensors are high-precision measurement devices which consist of electronic as well as mechanical silicon structures, designed for precision, efficiency, and mechanical robustness.

However, to achieve best possible results for your design, the following recommendations should be taken into consideration when mounting the sensor on a printed- circuit board (PCB).

It is generally recommended to keep a reasonable distance between the sensor mounting location on the PCB and the critical points described in the following examples. The exact value for a “reasonable distance” depends on many customer specific variables and must therefore be determined case by case on system level.

### 3.1 Push-button contacts

Keep a reasonable distance to push-button contacts, when placing the sensor device. Do not position the sensor directly beneath a push-button contact.

![Figure 1: Push-button contacts](images/p004-diagram-01.png)

> **Image description:** Block diagram of Push-button contacts.
>
> Components:
> push button contact
> sensor
> PCB
> base plate (e.g. shelf)
> anchor point (e.g. screw)
>
> Connections:
> The push button contact is situated on the base plate (e.g. shelf).
> The sensor is situated on the base plate (e.g. shelf) and is positioned away from the push button contact, with a "Keep distance !" indicator between them.
> The PCB is situated on the base plate (e.g. shelf).
> The anchor point (e.g. screw) is shown securing the assembly to the base plate (e.g. shelf) on both the left and right sides.
>
> Values:
> No voltage, frequency, part value, or other number with a unit is shown.

*Figure 1: Push-button contacts*

### 3.2 Thermal hot spots on the PCB

Keep a reasonable distance from any thermal hot spots, when placing the sensor device. Hot spots can for example be other integrated circuits with high power consumption.

![Diagram on page 5](images/p005-diagram-01.png)

> **Image description:** Block diagram of Thermal hot-spots on the PCB.
>
> Components:
> extremely hot spot (e.g. CPU)
> sensor
> PCB
> base plate (e.g. shelf)
> anchor point (e.g. screw)
>
> Connections:
> The extremely hot spot (e.g. CPU) is located on the PCB.
> The sensor is located on the PCB, maintaining a distance from the extremely hot spot (e.g. CPU).
> The PCB is mounted on the base plate (e.g. shelf).
> The PCB is secured to the base plate (e.g. shelf) via the anchor point (e.g. screw).
>
> Values:
> No voltage, frequency, part value, or other number with units is shown.

### 3.3 Distance to PCB anchor points

Keep a reasonable distance from any anchor points, where the PCB is fixed at a base plate (e.g. like a shelf or similar), when placing the sensor device.

![Diagram on page 5](images/p005-diagram-02.png)

> **Image description:** Block diagram of Figure 3: Distance to PCB anchor points.
>
> Components:
> - base plate (e.g. shelf)
> - anchor point (e.g. screw)
> - sensor
> - PCB
>
> Connections:
> - The base plate supports the PCB.
> - The anchor point connects to the PCB.
> - The sensor is positioned relative to the PCB and anchor point, with a dashed line indicating a required distance labeled "Keep distance!".
>
> Values:
> - No voltage, frequency, part value, or other number with units is shown.

### 3.4 Vibrations on the PCB

Do not place the sensor in areas where resonant amplitudes (vibrations) of the PCB are likely to occur or to be expected.

![Diagram on page 5](images/p005-diagram-03.png)

> **Image description:** Block diagram of Vibrations on the PCB.
>
> Components:
> base plate (e.g. shelf)
> anchor point (e.g. screw)
> sensor
> PCB with large span (e.g > 30mm)
>
> Connections:
> The base plate (e.g. shelf) supports the entire assembly.
> The anchor point (e.g. screw) is shown attached to the base plate.
> The sensor is placed on the PCB with large span (e.g > 30mm).
> A dashed line indicates a required distance between the sensor and the edge of the PCB with large span (e.g > 30mm), labeled "Keep distance!".
>
> Values:
> The PCB with large span has a minimum dimension of > 30mm.
>
> Figure caption:
> Figure 4: Vibrations on the PCB

### 3.5 Redundant PCB anchor points

It is recommended to unscrew or remove any redundant PCB anchor points. In theory, an ideal flat plane is determined exclusively by 3 anchor points. Any further anchor point will over- determine the ideal flat plane criteria. If these redundant anchor points are out of plane position (which means not 100% exact in plane position) the ideal flat criteria is infringed, resulting in mechanical stress.

The below given figure describes an expected stress maximum in the center of the diagonal crossover, assuming the 4 anchor points are not 100% exact in plane (over-determined ideal flat plane criteria). Unscrewing or removing one of the redundant anchor points can significantly minimize mechanical stress.

![Figure 5: redundant PCB anchor points](images/p006-diagram-01.png)

> **Image description:** Pinout or package drawing. Top view. Pin 1 is not explicitly indicated.
>
> Pins:
> 1: black circle (top left)
> 2: black circle (top right)
> 3: black circle (bottom left)
> 4: black circle (bottom right)
>
> Dimensions: No dimensions are shown.
>
> Text inside the figure: PCB, unscrew or, remove, anchor point, (e.g. screw).

*Figure 5: redundant PCB anchor points*

### 3.6 Mechanical stress maximum on the PCB

It is recommended to keep a reasonable distance from any mechanical stress maximum, when placing the sensor device. Mechanical stress can be induced for example by redundant anchor points, as described in 0. The below given example will show a stress maximum in the center of the diagonal crossover of the 4 anchor points. It is good manufacturing practice to always avoid or reduce the mechanical stress by optimizing the PCB design first, then to place the sensor in an appropriate low stress area.

![Diagram on page 6](images/p006-diagram-02.png)

> **Image description:** Block diagram illustrating the location of mechanical stress maximum on a PCB relative to a sensor placement.
>
> Components:
> PCB
> anchor point (e.g. screw)
> sensor
> Keep
> distance !
>
> Connections:
> The PCB is the main substrate.
> Four anchor points are located at the corners of the PCB.
> The sensor is located near the center of the PCB, where stress is maximum.
> The "Keep distance !" block is associated with the area around the sensor, indicating a recommended separation.
>
> Values:
> No specific voltage, frequency, part value, or unit is shown in the diagram.

### 3.7 Resin coatings

As shown in the figure below, ensure that the sensor is neither partially covered nor in contact with any (epoxy) resign material leading to an un-symmetric stress distribution over the sensor package.

![Diagram on page 7](images/p007-diagram-01.png)

> **Image description:** Block diagram of Figure 7: Resin coatings.
>
> Components:
> protected device (e.g. CPU)
> resin coating
> sensor
> PCB
> base plate (e.g. shelf)
> anchor point (e.g. screw)
>
> Connections:
> The protected device (e.g. CPU) is connected to the resin coating.
> The resin coating is shown covering the protected device (e.g. CPU) and extending over the sensor.
> The sensor is shown positioned above the PCB, and there is a red 'X' indicating it should not be covered by the resin coating.
> The PCB is mounted on the base plate (e.g. shelf).
> The anchor point (e.g. screw) is shown securing the PCB to the base plate (e.g. shelf).
>
> Values:
> No voltage, frequency, part value, or other number with units is shown.

### 3.8RF electric field sources

It is recommended to keep a reasonable distance from high-frequency electric field sources (especially above 2 GHz). Offset shifts and elevated noise may be observed for BMM350 during exposure to E-fields that depend on the strength and frequency of the electric field and the orientation of the field vector to the sensor.

Note that offset observed during E-field exposure scales quadratically with the electric field strength. For example, exposure to a 500 V/m field with a frequency in the range of 2.5 – 4 GHz may lead to offset drift of 1 uT, exposure to 1 kV/m to an offset drift of 4 uT.

### 3.9PCB surface finish

Do not use ENIG = Electroless Nickel / Immersion Gold (or called chemical Ni / Au), chemical Tin could be used.

## 4 PCB placement guideline based on magnetic influences

### 4.1 Non-geomagnetic field sources

The magnetic field measured by a geomagnetic sensor is a superposition of earth magnetic field and other environmental influences such as adjacent magnetic materials or fields induced by electric currents.

In this chapter, two other sources will be named, their strength will be compared to that of the geomagnetic field, and the implications for geomagnetic sensor applications will be explained.

The sources relevant for geomagnetic sensor applications are hard ferromagnetic materials and electric currents. Other sources do exist but are not relevant for geomagnetic sensor applications. Please note that the majority of disturbances stemming from the device (adjacent magnetic materials) are effectively compensated by the Sensor fusion software.

### On-board: Hard ferromagnetic materials

A hard magnetic material possesses a strong intrinsic magnetisation, which hardly reacts to external fields. These materials are typically used in speaker magnets and electrical motors (which are used for vibration alarm). Many different hard magnetic materials alloys exist. Some low-cost materials are ferrites (BaFeO, SrFeO, …) and AlNiCo; high-performance materials are for example SmCo and NdFeB.

The field strength of these fields is temperature dependent. For the popular low-cost ferrite magnets, the field decreases on rising temperature at a rate of typically 0.2 %/K; most other materials have a lower temperature coefficient. When operating a geomagnetic sensor in a ferrite-caused offset field of 500 µT, the offset will shift by 10 µT on a change of 10°C (18°F). If not detected, this will cause a major miscalculation of heading.

### On-board: Current-induced fields

Magnetic fields occur around electrical currents. The field direction follows the right -hand rule, which is depicted in Figure 8.

![Figure 8: The right-hand rule for current-induced magnetic fields](images/p008-picture-01.png)

> **Image description:** Rendering of the right-hand rule for current-induced magnetic fields.
>
> The image shows a circular loop with an arrow pointing upwards passing through its center. To the left of the loop, there is a label "B" next to an arrow pointing horizontally to the left. To the right of the loop, there is a label "I" next to an arrow pointing horizontally to the right.
>
> No dimensions or scale are shown.

*Figure 8: The right-hand rule for current-induced magnetic fields*

The absolute magnetic field strength B at a given point outside of the wire depends on the current flowing through the wire and the distance to its centre according to the following formula:

$$
B = \mu_0 \cdot \mu_r \frac{I}{2\pi r} \text{ where}
$$

$\mu_r =$ relative permeability $\approx 1$ for air, and

$\mu =$ vacuum permeability $= 4\pi \cdot 10^{-7} \text{ H m/I}$

Equation 1

As an example, a power line on a PCB could carry a current of 100 mA. At a distance of 5 mm from this line, the magnetic field strength equals:

$$
B = \mu_0 \cdot \mu_r \cdot \frac{I}{2\pi r} = 4\pi \cdot 10^{-7} \frac{H}{m} \cdot 1 \cdot \frac{100\text{mA}}{2\pi \cdot 5\text{mm}} = 4\mu\text{T}
$$

Equation 2

As a typical value for the horizontal geomagnetic field strength is 20 µT, a current-induced field can strongly influence heading calculation results, so that some distance must be kept. However, this is only the case for high-current lines, signals lines (e.g. an SPI bus) do not cause distortions.

### Comparison of geomagnetic field strength to other fields

The geomagnetic field is very weak compared to other sources of magnetism. A typical speaker magnet can generate a field of 11,000 µT, which by far exceeds the strength of the earth magnetic field. The calculation above shows that, at least at close distances, current-induced fields can generate fields with strengths similar to the geomagnetic field.

### 4.2 Distortion and shielding by soft magnetic materials

The measurement of the earth magnetic field is not only affected by the existence of the other field sources named in chapter 5.1, but also the materials that can distort the surrounding fields. These materials are called soft magnetic materials. Unlike hard magnetic materials, these materials easily change their magnetization based on the surrounding fields and attract the magnetic field lines. This leads to two phenomena: distortion and shielding.

### On-board: Distortion

For a geomagnetic sensor, distortion is the effect that the field lines near soft magnetic materials are ‘bent’, so that their direction is changed from large-scale geomagnetic field direction. An example is given in Figure 9, where this field visible to the sensor is not pointing in the direction of magnetic north anymore. In addition, many soft magnetic materials are anisotropic, which means that the behavior depends on the direction of the field applied.

![Figure 9: Magnetic field distortion due to soft iron effects](images/p009-diagram-01.png)

> **Image description:** Block diagram of Magnetic field distortion due to soft iron effects.
>
> Components:
> Soft iron
> Sensor
>
> Connections:
> Geomagnetic field lines connect to the Soft iron.
> Geomagnetic field lines connect to the Sensor.
>
> Numbers:
> None.

*Figure 9: Magnetic field distortion due to soft iron effects*

Software compensation of this effect is very difficult. For maritime applications, geomagnetic sensors are calibrated to correct the soft magnetic effects of the ship’s iron. If no precise calibration is possible, such as in consumer applications, soft magnetic materials should be avoided in the vicinity of geomagnetic sensors.

### On-board and external: Shielding

If a space is surrounded by soft magnetic material (solid or framework), this surrounding will absorb the magnetic field lines. This is depicted in Figure 10. As a result, there will be virtually no remaining field within the structure. If this effect is desired, the materials like the commercially available Mu-metal can be used as magnetic shielding.

![Picture on page 10](images/p010-picture-02.png)

> **Image description:** Rendering of a cross-section showing magnetic field lines around a circular component.
>
> The object shown is a circular component, likely a magnetic material, viewed from the side.
>
> Visible markings include:
> - Three vertical blue lines inside the circular component, representing internal structures or magnetic poles.
> - Several colored lines (yellow, red, cyan, light blue) forming concentric or curved patterns around the internal blue lines, representing magnetic field lines or flux paths within the component.
> - Numerous external blue lines representing magnetic field lines emanating from and surrounding the circular component.
>
> No dimensions or scale are shown.

![Figure 10: Magnetic shielding within a soft magnetic sphere (source: Institute for Theory of electrical Engineering, Universität Stuttgart)](images/p010-figure-01.png)

> **Image description:** Block diagram of magnetic shielding within a soft magnetic sphere.
>
> Components:
> A central sphere structure.
> Several internal vertical lines within the sphere, colored blue, red, and yellow.
> External curved lines surrounding the sphere.
>
> Connections:
> The external curved lines appear to be connected to the central sphere structure, with arrows indicating flow or influence radiating outwards from the sphere.
> The internal vertical lines are contained within the central sphere structure.
>
> Numbers:
> No voltage, frequency, part value, or other numbers are shown.

*Figure 10: Magnetic shielding within a soft magnetic sphere (source: Institute for Theory of electrical Engineering, Universität Stuttgart)*

A geomagnetic sensor cannot work without the external magnetic field. Hence, the better the shielding is, the less signal is available for the geomagnetic sensor to work with. Especially the iron frame of buildings can be an unwanted cause of magnetic shielding. This explains why geomagnetic sensor performance can suffer within a building containing steel.

### Overview of external influences

In this chapter, many possible external influences to geomagnetic sensor performance have been discussed. For a better overview, the impact on geomagnetic sensor input will now be given in Figure 11, in which the field in X and Y direction over a 360° geomagnetic sensor rotation is depicted. A centered circle depicts ideal geomagnetic sensor input; the distortions are clearlyvisible.

![Figure 11: Overview of the effects of external influences on geomagnetic sensor input](images/p010-diagram-03.png)

> **Image description:** Block diagram of the effects of external influences on geomagnetic sensor input.
>
> Components:
> Ideal field
> Fluctuating offsets due to currents
> Offset due to hard magnet
> Distortion due to soft magnet
> Shielding
>
> Connections:
> The Ideal field shows a circle centered at the origin with axes labeled BX and BY.
> The Fluctuating offsets due to currents shows a spiral pattern centered at the origin with axes labeled BX and BY.
> The Offset due to hard magnet shows a circle centered away from the origin along the BY axis with axes labeled BX and BY.
> The Distortion due to soft magnet shows an ellipse centered near the origin with axes labeled BX and BY.
> The Shielding shows a small circle centered at the origin with axes labeled BX and BY.
>
> Numbers:
> No voltage, frequency, part value, or other numbers with units are shown.

*Figure 11: Overview of the effects of external influences on geomagnetic sensor input*

### 4.3 Possibilities and limitations of software compensation

The effects mentioned in the previous chapter can partially be corrected by software.

**On-board hard magnetic offsets** are almost unavoidable in most of the environment and therefore it needs a calibration phase. In this calibration phase, the offset is estimated so that it can be subtracted from further measurements. This initial calibration is depicted in Figure 12

![Figure 12: Offset estimation in calibration phase.](images/p011-figure-01.png)

> **Image description:** Graph of data points gathered during calibration phase and corrected data after offset estimation.
>
> The horizontal axis is labeled BX and has tick labels ranging from unreadable to unreadable.
> The vertical axis is labeled BY and has tick labels ranging from unreadable to unreadable.
>
> The first series is labeled "Data points gathered during calibration phase" and consists of blue 'x' markers forming a rough circle centered away from the origin.
> The second series is labeled "Corrected data after offset estimation (around origin)" and consists of blue 'x' markers forming a rough circle centered near the origin.
>
> Text printed in the figure includes:
> Data points gathered during calibration phase
> BX
> BY
> BX
> BY
> Estimated offset
> Measured data
> Corrected data after offset estimation (around origin)

*Figure 12: Offset estimation in calibration phase.*

However, if the hard magnetic offsets exceed the sensor range, the sensor will become saturated, and heading cannot be measured anymore.

**The temperature dependence** of the hard magnetic offsets will cause the originally estimated offset to change when temperature changes. If the sensor does not see many different orientations after the original calibration, this change may go unnoticed and result in a significant error (e.g. ~15° heading error on 5°C change of a 500 µT offset magnet).

Figure 13 illustrates this. The center of the blue circle is the offset caused by permanent magnets. The radius of the blue circle equals the magnitude of the earth magnetic field.

The blue x marks the measured field strength. This is going directly in +y direction from the circle’s center. The correct heading can only be estimated after the circle’s center has been moved to the axes’ origin (second image). When the offset changes (third image), while the offset compensation still subtracts the same value, the estimated heading is again wrong.

![Figure 13: Errors caused bychanging magnetic offsets](images/p011-figure-02.png)

> **Image description:** Block diagram illustrating errors caused by changing magnetic offsets.
>
> Components:
> - Blue circle (representing offset)
> - Orange arrow (representing Estimated heading)
> - Blue arrow (representing Magnetic heading)
> - Dashed blue circle (representing Earth magnetic field)
> - Blue arrow (representing Measured field)
> - Blue arrow (representing Offset correction)
>
> Connections:
> - In the first panel, the blue circle is positioned such that the orange arrow (Estimated heading) is not aligned with the blue arrow (Magnetic heading).
> - In the second panel, the blue circle is shifted, and the orange arrow (Estimated heading) is aligned with the blue arrow (Magnetic heading) after the application of the Offset correction.
> - In the third panel, the blue circle is shifted again, and the orange arrow (Estimated heading) is again not aligned with the blue arrow (Magnetic heading) after the Offset correction.
>
> Numbers:
> No voltage, frequency, part value, or other numbers with units are shown.

*Figure 13: Errors caused bychanging magnetic offsets*

**On-board current-induced fields** change unpredictably (e.g., on display brightness change, during network activity …). Because of this, their effects cannot be compensated, and the fields will appear to add small varying offsets (appearing like noise) or large offsets to the sensor signal.

**On-board soft magnetic effects** are very difficult to compensate for. Since soft magnetic materials can distort or bend the geomagnetic field in different directions, depending on temperature, field strength and magnetic history of the material involved, a very large data set would be required to estimate all magnetic effects.  
For these reasons, soft magnetic materials must (and can) be avoid ed in the vicinity of a geomagnetic sensor instead of being compensated in software.

**On-board or external shielding** due to on-board soft magnetic cover or external iron in buildings can attenuate the earth magnetic field so much that it becomes almost immeasurable. External effects cannot be estimated by the compass software and therefore cannot be compensated. On-board shielding will still dampen the signal so much that reliable heading estimation becomes impossible

### 4.4PCB placement advice

To achieve optimum geomagnetic sensor performance, several rules need to be kept in order to avoid the disturbances mentioned above.

### 4.5 Hard magnetic offsets

Hard magnetic offsets must be as limited as much as possible. Ideally, the offset should bebelow 100 µT, maximum 200 µT. If this is not possible, please refer to the following two rules: **Sensor range may not be** **exceeded over entire temperature and magnet variation range.** To keep to this rule, the offset at the proposed location should be measured. Then,using information about sample spread and temperature dependence of the magnet(s) thatcause(s) this field, the effects over temperature can be estimated.  
Example: A ferrite magnet has a temperature dependence of -0.2 %/K. A field of 600 µT measured at 20 °C will grow to (-0.002 x (-40 – 20) + 1) x 600 = 1.12 x 600 = 672 µT at a temperature of -40 °C. To accommodate for magnet variance as well, we recommend not using more than 75 % of the magnetic sensor range at room temperature. This results in the following rule:

$$
B_x, B_y, B_z < 0.75 \cdot 2400 \mu T
$$

Equation 3

Not following this rule would result in certain conditions, wherein the sensor range will be exceeded, and geomagnetic sensor functionalitywill be lost.

**External field temperature effects must be minimized.** Even if the range of the sensor is large enough to cope with the offset present, its temperature dependence will still impact geomagnetic sensor performance. Recommendation is to have no more than 200µT absolute field strength (i.e., √(X²+Y²+Z²)) assuming a temperature coefficient of -0.2 %/K. if the temperature coefficient is lower, the value of 200µT can be increased accordingly (e.g., 0.4mT for a coefficient of 0.1%/K). This results in the following rule:

$$
\sqrt{B_x^2 + B_y^2 + B_z^2} \quad \cdot TC_{\text{magnet}} < 200 \mu\text{T} \cdot 0.2\%/\text{K}
$$

*Equation 4*

Not following this rule can result in noticeable temperature dependence of the geomagnetic sensor.

### PCB Currents

Constant currents on the PCB cause a constant offset on the sens or signal and are compensated by software. However, if fluctuating currents are routed closely to the magnetic sensor, then the changing field strength caused by such a current should not exceed 0.5 µT, while lower numbers are better. Following equation 5, for a given current Ifluc, the required distance equals:

$$
r = \frac{\mu_0 \cdot \mu_r}{2\pi \cdot B} \cdot I_{fluc} = 0.4 \cdot I_{fluc} , \text{ using}
$$

$\mu_r$ = relative permeability $\approx 1$ for air,  
$\mu_0$ = vacuum permeability = $4\pi \cdot 10^{-7} \text{ H} \cdot \text{m}$  
$I_{fluc}$ = fluctuating current, and  
$B$ = allowed fluctuating field strength ($0.5 \mu T$)

*Equation 5*

Please use the following table to estimate the required distance between the current line andthe geomagnetic sensor:

*Table 1: Required distance between magnetic sensor and current lines*

| <b>Current change (max-min)[mA]</b> | <b>Required distance [mm]</b> |
| --- | --- |
| 1 | 0.4 |
| 2 | 0.8 |
| 5 | 2.0 |
| 10 | 4.0 |
| 20 | 8.0 |
| 50 | 20.0 |

### Soft magnetic effects

The usage of soft magnetic materials close to the geomagnetic sensor must be avoided. The presence of soft magnetic materials at the sensor can be estimated by rotating the PCB on a nonmagnetic table (free of iron). Ideally, the measured data points will all be in a circle. In usual cases, soft magnetic effects are visible as distortion of this circle into an ellipse as visible in Figure 14.

![Figure 14: Quantifying soft magnetic distortion](images/p014-figure-01.png)

> **Image description:** Graph of magnetic field components.
> The horizontal axis is labeled BX and has no units or tick labels visible.
> The vertical axis is labeled BY and has no units or tick labels visible.
>
> There are two main plots:
> The left plot is labeled "Ideal measurement data on a circle". It shows "Measured data" represented by blue 'x' marks forming a circle.
> The right plot is labeled "Anisotropic distortion due to soft magnetic effects". It shows a shape distorted from a circle into an ellipse. This shape is composed of "Measured data" (blue 'x' marks), a "Major ellipse" (dashed line), and a "Minor ellipse" (solid orange line).
>
> Text printed in the figure includes:
> "Ideal measurement data on a circle"
> "BX"
> "BY"
> "BX"
> "BY"
> "Major ellipse"
> "axis"
> "Measured"
> "data"
> "Anisotropic distortion due to soft magnetic effects"
> "Minor ellipse"
> "axis"

*Figure 14: Quantifying soft magnetic distortion*

If the distortion can be approximated by an ellipse, the ratio between major and minor ellipse axis defines the soft magnetic effect. The mismatch should be below 3 %, resulting in the last rule:

$$
\frac{L_{major\_axis}}{L_{minor\_axis}} < 1.03
$$

*Equation 6*

### Overview

An overview of the previously presented rules is shown in the table below

*Table 2: Overview of placement guidelines*

<table>
<thead>
<tr><th colspan="2"><b>Hard magnetism</b></th></tr>
<tr><th>Ideal:</th><th>External field &lt; 100 μT</th></tr>
<tr><th>Second best:</th><th>External field &lt; 200 μT</th></tr>
<tr><th>Otherwise:</th><th>External field &lt; 0.75 x sensor range (room temperature), and $\sqrt{(BX^2+BY^2+BZ^2)} \times T\text{Cext. field} < 0.40\text{mT} \times \%/\text{K}$<br>xTCext.<br>0.40mT<br>x%/K</th></tr>
<tr><th colspan="2"><b>Fluctuating PCB currents</b></th></tr>
</thead>
<tbody>
<tr><td colspan="2">Distance $r$ [mm] &gt; 0.4 x Ifluc [mA]</td></tr>
<tr><th colspan="2"><b>Soft magnetism</b></th></tr>
<tr><td colspan="2">Ratio major axis / minor axis &lt; 1.03</td></tr>
</tbody>
</table>

## 5 PCB layout recommendation

### 5.1 Dimensions and landing pattern recommendation

Find information on the landing pattern and sensor dimensions below.

![Diagram on page 15](images/p015-diagram-02.png)

> **Image description:** Package drawing, TOP VIEW. Pin 1 is indicated in the TOP VIEW.
>
> Pins:
> 1: 1
> 2: 2
> 3: 3
> A: A
> B: B
> C: C
>
> Dimensions:
> 0.5 ±0.04 (0.194)
> 0.306 ±0.025 (0.025)
> 0.075 C
> 0.05 C A B
> 0.015 C
> Ø0.269 ±0.03 (9x)
> (0.24)
>
> Package drawing, BOTTOM VIEW. Pin 1 is indicated in the BOTTOM VIEW.
>
> Pins:
> 1: 1
> 2: 2
> 3: 3
> A: A
> B: B
> C: C
>
> Dimensions:
> (1.28)
> 0.64
> 0.8
> 0.4
> 6x
> (4x)
> (1.28)
> 0.03 A B C (4x)
> 0.64
> 0.4
> 0.8
> (0.24)

![Figure 15: BMM350 package dimensions](images/p015-figure-01.png)

> **Image description:** Figure 15: BMM350 package dimensions.
>
> Top View:
> The package is shown in a TOP VIEW. Pin 1 is indicated by A1-MARKER.
> Pins are numbered 1, 2, and 3.
> Dimensions shown are: 0.5 ±0.04 (0.194), 0.306 ±0.025 (0.025), and 0.075 C.
>
> Bottom View:
> The package is shown in a BOTTOM VIEW.
> Pins are indicated by A, B, and C.
> Pin details include:
> 0.05 C A B
> 0.015 C
> Ø0.269 ±0.03 (9x)
> Dimensions shown are: (0.24), 0.64, 0.8, 0.4, 6x, (4x), (1.28).
>
> Pin order and names:
> 1: (No name provided)
> 2: (No name provided)
> 3: (No name provided)
> A: (No name provided)
> B: (No name provided)
> C: (No name provided)
>
> Dimensions:
> 0.5 ±0.04 (0.194)
> 0.306 ±0.025 (0.025)
> 0.075 C
> 0.05 C A B
> 0.015 C
> Ø0.269 ±0.03 (9x)
> (0.24)
> 0.64
> 0.8
> 0.4
> 6x
> (4x)
> (1.28)

*Figure 15: BMM350 package dimensions*

![Diagram on page 15](images/p015-diagram-04.png)

> **Image description:** Pinout or package drawing. Top view. Pin 1 is not explicitly indicated.
>
> There are 9 pins arranged in a 3x3 grid. The pins are represented by circles. No pin names are provided.
>
> Dimensions:
> 1.28 (vertical dimension)
> 0.4 (vertical spacing between rows)
> 1.28 (horizontal dimension)
> 0.4 (horizontal spacing between columns)
> Ø0.25(9x) (pin diameter)

![Figure 16: Recommended landing pattern for soldering to PCB.](images/p015-figure-03.png)

> **Image description:** Pinout or package drawing. Top view. Pin 1 is not explicitly indicated.
>
> There are 9 pins arranged in a 3x3 grid. The pins are not numbered or named in the drawing.
>
> Dimensions:
> 1.28 (vertical dimension)
> 0.4 (vertical spacing)
> 1.28 (horizontal dimension)
> 0.4 (horizontal spacing)
> Ø0.25(9x) (pin diameter)

*Figure 16: Recommended landing pattern for soldering to PCB.*

### 5.2 Connection diagram

![Figure 17: BMM350 connection diagram (TOP view).](images/p016-diagram-01.png)

> **Image description:** Pinout or package drawing of BMM350 connection diagram (TOP view). Pin 1 is not explicitly indicated.
>
> Pins:
> A3: A3
> A2: A2
> A1: A1
> B3: B3
> B2: B2
> B1: B1
> C3: C3
> C2: C2
> C1: C1
> BYPASS: BYPASS
> GND: GND
> VDD: VDD
> CRST: CRST
> VDDIO: VDDIO
> SCK: SCK
> SDA: SDA
> INT: INT
> ADSEL: ADSEL
>
> Dimensions:
> 100nF (capacitor)
> 2.2uF (capacitor)
> 100nF (capacitor)
>
> Other visible text:
> ADSEL status defines I2C address LSB
> Lasermarking side, balls not visible
> x
> y

*Figure 17: BMM350 connection diagram (TOP view).*

- 100nF capacitors should be used for VDD and VDDIO decoupling.
- CRST requires a 2.2uF capacitor (200mA peak current, need low resistance, low inductance type with nonmagnetic properties). For part recommendations please refer to the BM350 data sheet.

### 5.3 Layout recommendations

The central ADSEL pin can be directly routed to GND or VDDIO for address selection. It can also be connected externally using a blind via in case address selection is desired, however, large vias and a mixed VIPP / non-VIPPO design should be avoided, as the thermal expansion of the via during soldering can lead to a risk of voiding in the center pin.

**The following peak current can flow:**

- 400mA current ( very short time< 1us ) between CRST capacitor (2.2uF) and CRST pin
- 400mA current ( very short time< 1us ) between CRST capacitor (2.2uF) and GND pin
- 20mA max current ( short time, < 10ms ) in VDD pin, 750uA maximum average current

## 6 Reflow Soldering & Moisture Sensitivity Level

To ensure that sensors are properly mounted it is important to avoid mechanical stress by ensuring a proper solder thickness after soldering. Furthermore, solder paste thickness after soldering should be as uniform as possible to reduce stress.

The **moisture sensitivity level** (MSL) of the device corresponds to JEDEC **Level 1**, see also

- IPC/JEDEC J-STD-020D "Joint Industry Standard: Moisture/Reflow Sensitivity Classification for non-hermetic Solid State Surface Mount Devices"
- IPC/JEDEC J-STD-033D "Joint Industry Standard: Handling, Packing, Shipping and Use of Moisture/Reflow Sensitive Surface Mount Devices"

For Qualification of the sensor, the reflow profile according to JEDEC J-STD020 D is used. This profile has proven to work well with the geometry and ball size of the part.

The used solder paste in the standard process is a lead-free no-clean solder paste, with a stencil height of 80 µm. Depending on the type of solder paste used, residues of solder flux after the assembly of BMM on PCB may affect the sensor performance under temperature cycling. In this case, PCB cleaning is highly recommended to remove the residues of solder flux.

Note that the sensor performance was characterized **without** application of underfill, glue or encapsulation. Changes in performance may occur in case an underfill, glue or encapsulation is used, especially under strain and/or thermal stress.

The product can withstand in total up to 3 reflow soldering cycles. This could be a situation where a PCB is mounted with devices from both sides (i.e., 2 reflow cycles necessary) and in the next step an additional re-work cycle could be required (1 reflow).

## 7 Wafer Level Chip Scale Package Handling

Since BMM products in Wafer Level Chip Scale Package have no EMC mold protection they should be handled with great care. The devices should remain secure in their carriers except during unloading or inspection operations at dedicated ESD-safe workplaces. WLCSP parts should not be handled with bare hands, even wearing gloves or finger cots.

In cost driven assembly builds like low volume productions and engineering runs, manual handling of the devices may be necessary. In these cases, it is mandatory that tweezers of any form are **not** used to pick up, transfer or place the WLCSP. Tweezers often cause mechanical damage to the WLCSP, either to the top surface or to the silicon. Scratches, chip-out, and micro-cuts are hard to avoid using tweezers of any form. This kind of damage maybe visible but often does not manifest until electrical testing. Therefore, it is recommended that any manual handling is done by using suitable designed tools such as vacuum pencils with PTFE tips for sawn-edge pick-up.

For automatic pick and place equipment a soft silicon/rubber tip of suitable size should be used. These tips should be inspected and cleaned in well-defined periods to avoid mechanical damage to the chip. The operating pressure of the pick and place tool should be minimized.

## 8 Handling of reels

### 8.1 Storage

Once the reels are removed from the pizza box, they should always be stacked in vertical condition.

### 8.2 Introduction into production

Reel trailers must not be removed. Removal of the trailer could cause deformation of the reel during de-reeling and consequently tilted parts

![Figure 18: Reel trailers must not be removed](images/p018-picture-01.png)

> **Image description:** Photo of a reel trailer being removed from a reel, viewed from the side.
>
> The image shows a black reel with a trailer attached. A red arrow points to the area where the trailer has been removed. A large red 'X' is present on the right side of the image.
>
> Text inside the figure reads: Reel trailer was removed!
>
> No dimensions or scale are shown.

*Figure 18: Reel trailers must not be removed*

Reels must be stored vertically as shown in the image below

![Figure 19: Reels must be stored vertically as shown](images/p018-picture-02.png)

> **Image description:** Photo showing two views of a reel of electronic components in a storage tray.
>
> Top view: A reel of black components is stored horizontally in a black plastic storage tray. The tray has multiple slots. A green checkmark and the text "Correct Storage." are visible to the right of the tray.
>
> Bottom view: A reel of black components is stored vertically in a black plastic storage tray. The reel has a white label affixed to it. A red 'X' and the text "Wrong Storage." are visible to the right of the tray.
>
> No dimensions or scale are shown.

*Figure 19: Reels must be stored vertically as shown*

When reels are to be stored horizontally, they must be ordered in a stack with at most 4 reels per stack (see images below).

![Figure 20: Stacking of reels](images/p019-picture-01.png)

> **Image description:** Photo of two views of a reel storage setup.
>
> Top view: A black plastic tray containing a single reel of components. The reel has a paper label affixed to it. To the right of the tray, there is a green checkmark followed by the text "Correct Storage.".
>
> Bottom view: A black plastic tray containing a stack of three reels of components. The top reel has a paper label. The middle and bottom reels also have paper labels. A red arrow points to the stack of reels. To the right of the tray, there is a red 'X' followed by the text "Wrong Storage.".
>
> No dimensions or scale are shown.

*Figure 20: Stacking of reels*

### 8.3 Orientation of sensors on the reel

The orientation of the sensors on the reel is shown in Figure 21.

![Figure 21: Orientation of sensors on the reel. Pin 1 (“INT”) marker at upper left position](images/p019-picture-02.png)

> **Image description:** Rendering of a tape reel from the Tape Top Side.
>
> Visible markings include:
> Carrier
> Sprocket Hole
> Pocket
> Tape Top Side
>
> No dimensions or scale are shown.

*Figure 21: Orientation of sensors on the reel. Pin 1 (“INT”) marker at upper left position*

### 8.4 Reel Dimensions

**Notes:**

(\*) 10 sprocket hole pitch cumulative tolerance ±0.2

(\*\*) Pocket position relative to sprocket hole measured as true position of pocket, not pocket hole

![Figure 22: Reel dimensions](images/p020-diagram-01.png)

> **Image description:** Reel dimensions drawing.
>
> Pinout is not shown.
>
> Pins are not shown.
>
> Dimensions:
> 2.00±0.05 (**)
> 4.00 (*)
> Ø 1.50+0.1-0.0
> 1.75±0.10
> 3.50±0.05 (**)
> 8.00+0.3-0.1
> (1.6)
> (5°)
> Ao 1.41 0.05
> Bo 1.41 0.05
> Ko 0.64 0.05
>
> Sections shown:
> Section B ↔ B
> Section A ↔ A

*Figure 22: Reel dimensions*

## 9 Returning WLCSP for Failure Analysis

WLCSPs can be easily damaged while removing the part from the assembly either because of de-soldering (thermal stress) or handling (mechanical stress).

In case a part needs to be returned to BST for failure analysis (FA) purposes, please send the PCB on which the part is assembled. If not possible, carefully cut a larger area around the part while avoiding warpage of the assembly and return it to BST. For shipment, use material and packaging that protects against electro-static discharge (ESD) and mechanical damage, e.g., place the part into an ESD bag, wrap the bag in foam material and ship in a stable package.

This procedure is required to ensure that the FA can be conducted at BST.

Note: If any underfill or encapsulation was used (not recommended, see Chapter 6) and the part needs to be returned to BST for FA, please provide details on the type of material used for the underfill/ encapsulation (silicone based etc.) to facilitate chemical removal at BST for FA.

In case the part must be removed from the PCB, please get in contact with BST to receive a guideline on how to safely de-solder WLCSPs.

## 10 Environmental Safety

### 10.1 RoHS

The BMM350 sensor meets the requirements of the EC restriction of hazardous substances (RoHS) directive, see also: Directive 2002/95/EC of the European Parliament and of the Council of 8 September 2011 on the restriction of the use of certain hazardous substances in electrical and electronic equipment.

### 10.2 Halogen content

The BMM350 is halogen-free. For more details on the analysis results please contact your Bosch Sensortec representative.

## 11 Legal Disclaimer

### 11.1 Engineering samples

Engineering Samples are marked with an asterisk (\*), (e) or (E). Samples may vary from the valid technical specifications of the product series contained in this data sheet. They are therefore not intended or fit for resale to third parties or for use in end products. Their sole purpose is internal client testing. The testing of an engineering sample may in no way replace the testing of a product series. Bosch Sensortec assumes no liability for the use of engineering samples. The Purchaser shall indemnify Bosch Sensortec from all claims arising from the use of engineering samples.

### 11.2 Product use

Bosch Sensortec products are developed for the consumer goods industry. They may only be used within the parameters of this product data sheet. They are not fit for use in life-sustaining or safety-critical systems. Safety-critical systems are those for which a malfunction is expected to lead to bodily harm, death or severe property damage. In addition, they shall not be used directly or indirectly for military purposes (including but not limited to nuclear, chemical or biological proliferation of weapons or development of missile technology), nuclear power, deep sea or space applications (including but not limited to satellite technology).

The resale and/or use of Bosch Sensortec products are at the purchaser’s own risk and his own responsibility. The examination of fitness for the intended use is the sole responsibility of the purchaser.

The purchaser shall indemnify Bosch Sensortec from all third-party claims arising from any product use not covered by the parameters of this product data sheet or not approved by Bosch Sensortec and reimburse Bosch Sensortec for all costs in connection with such claims.

The purchaser accepts the responsibility to monitor the market for the purchased products, particularly with regard to product safety, and to inform Bosch Sensortec without delay of all safety-critical incidents.

### 11.3 Application examples and hints

With respect to any examples or hints given herein, any typical values stated herein and/or any information regarding the application of the device, Bosch Sensortec hereby disclaims any and all warranties and liabilities of any kind, including without limitation warranties of non-infringement of intellectual property rights or copyrights of any third party. The information given in this document shall in no event be regarded as a guarantee of conditions or characteristics. They are provided for illustrative purposes only and no evaluation regarding infringement of intellectual property rights or copyrights or regarding functionality, performance or error has been made.

## 12 Document history and modification

| <b>Rev. No</b> | <b>Chapter</b> | <b>Description of modification/changes</b> | <b>Date</b> |
| --- | --- | --- | --- |
| 1.0 | all | Initial release | September 2021 |
| 1.3 | all | Revision | April 2023 |
| 1.4 | 8.3 | Orientation of sensor in reel | May 2023 |
| 1.5 | 5.1 | Change package picture | November 2024 |
| 1.6 | 4.5, 4.53 Table 2 | Changed external field | June 2025 |
| 1.7 | 6 | Recommendation of PCB cleaning | July 2025 |
| 1.8 | 3.9 | Added PCB surface finish | April 2026 |

**Bosch Sensortec** | BST-BMM350-HS000-08 24 | 24

© Bosch Sensortec GmbH 2023 | All rights reserved, also regarding any disposal, exploitation, reproduction, editing, distribution, Document number: BST-BMM350-HS000-08 as well as in the event of applications for industrial property rights

**Bosch Sensortec GmbH** Gerhard-Kindler-Straße 9 72770 Reutlingen / Germany contact@bosch-sensortec.com www.bosch-sensortec.com

Modifications reserved

Preliminary - specifications subject to change without notice Document number: BST-BMM350-HS000-08
