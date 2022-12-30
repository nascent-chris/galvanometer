import matplotlib.pyplot as plt
import numpy as np
from scipy.optimize import curve_fit


def func(x, a, b, c):
    return a*x**2 + b*x + c


# Define the data
measured_gau = np.array([0, 108, 171, 242, 299])
duty_cycle = np.array([36, 88, 115, 141, 160])

# Fit the curve to the data
params, params_covariance = curve_fit(func, measured_gau, duty_cycle)

# Print the fitted curve equation
print(f'y = {params[0]:.5f}x^2 + {params[1]:.5f}x + {params[2]:.5f}')
print(
    f'let a = {params[0]:.5f}\nlet b = {params[1]:.5f}\nlet c = {params[2]:.5f}')


# Plot the data and the fitted curve
plt.plot(measured_gau, duty_cycle, 'o', label='data')
plt.plot(measured_gau, func(measured_gau, params[0], params[1], params[2]),
         label='fit')
plt.legend(loc='best')
plt.show()
