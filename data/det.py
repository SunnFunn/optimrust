import numpy as np


A = np.array([[1, 1, 0, 0, 0], [0, 0, 1, 1, 0], [1, 0, 0, 0, 0], [0, 0, 1, 0, 0], [0, 1, 0, 1, 1]])
res = np.linalg.det(A)

if __name__ == "__main__":
    print(res)