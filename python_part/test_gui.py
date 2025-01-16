import tkinter as tk
from tkinter import ttk
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
from matplotlib.figure import Figure
import random
import time

class RealTimeScatterApp:
    def __init__(self, root):
        self.root = root
        self.root.title("Real-Time Scatter Plot")

        # Create a frame for the graph
        self.graph_frame = ttk.Frame(self.root)
        self.graph_frame.pack(fill=tk.BOTH, expand=True)

        # Create a matplotlib figure and axis
        self.fig = Figure(figsize=(5, 4), dpi=100)
        self.ax = self.fig.add_subplot(111)
        self.ax.set_title("Real-Time Scatter Plot")
        self.ax.set_xlabel("X Coordinates")
        self.ax.set_ylabel("Y Coordinates")

        # Add the figure to the Tkinter canvas
        self.canvas = FigureCanvasTkAgg(self.fig, self.graph_frame)
        self.canvas.get_tk_widget().pack(fill=tk.BOTH, expand=True)

        # Initialize data for the scatter plot
        self.x_data = []
        self.y_data = []
        self.color_value = 0.5  # Default color value

        # Start the update loop
        self.update_graph()

    def update_graph(self):
        # Simulate adding new data
        self.add_new_data()

        # Clear and redraw the scatter plot
        self.ax.clear()
        scatter = self.ax.scatter(self.x_data, self.y_data, c=[self.color_value] * len(self.x_data), cmap='viridis', s=50)
        self.ax.set_title("Real-Time Scatter Plot")
        self.ax.set_xlabel("X Coordinates")
        self.ax.set_ylabel("Y Coordinates")

        # Refresh the canvas
        self.canvas.draw()

        # Schedule the next update
        self.root.after(1000, self.update_graph)  # Update every 1 second

    def add_new_data(self):
        # Example: Add random data to simulate updates
        self.x_data.append(random.uniform(0, 10))
        self.y_data.append(random.uniform(0, 10))

        # Limit the data to 50 points for clarity
        if len(self.x_data) > 50:
            self.x_data.pop(0)
            self.y_data.pop(0)

    def set_data(self, x_data, y_data, color_value):
        """Set the data for the scatter plot."""
        self.x_data = x_data
        self.y_data = y_data
        self.color_value = color_value
        self.update_graph()  # Immediately update the graph with new data

if __name__ == "__main__":
    root = tk.Tk()
    app = RealTimeScatterApp(root)

    # Example usage: Set data after a short delay
    def set_example_data():
        x = [random.uniform(0, 10) for _ in range(30)]
        y = [random.uniform(0, 10) for _ in range(30)]
        color = 0.8  # Example color intensity
        app.set_data(x, y, color)

    root.after(2000, set_example_data)  # Set data after 2 seconds
    root.mainloop()
