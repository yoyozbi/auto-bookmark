# Use the official Python 3.8 slim image as the base image
FROM python:3.11-slim
ARG TIMEOUT=180 # Timeout for the gunicorn server
ARG WORKER_NUMBER=4 # Number of workers for the gunicorn server

# Set the working directory within the container
WORKDIR /app

# Copy the necessary files and directories into the container
COPY  requirements.txt .
RUN pip3 install --upgrade pip && pip install --no-cache-dir -r requirements.txt
RUN pip install gunicorn
RUN apt-get update && apt-get install -y poppler-utils

COPY src/ .
# Upgrade pip and install Python dependencies

# Expose port 5000 for the Flask application
EXPOSE 5000

# Define the command to run the Flask application using Gunicorn
ENV TIMEOUT=$TIMEOUT
ENV WORKER_NUMBER=$WORKER_NUMBER
CMD ["gunicorn", "main", "-b", "0.0.0.0:5000", "-w", "$WORKER_NUMBER", "--timeout", "$TIMEOUT"]
